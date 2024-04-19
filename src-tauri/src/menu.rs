use serde::Serialize;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::Shell::*;
use windows_core::{PCWSTR, PWSTR};
use windows::Win32::UI::WindowsAndMessaging::*;
use once_cell::sync::Lazy;
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::Mutex;
use crate::settings::Theme;
use crate::util::WM_APPTHEMECHANGE;
use crate::util::{Counter, encode_wide, decode_wide};

const INITIAL_MODE:i32 = 3;
static MODE:Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(INITIAL_MODE));
static HAS_THEME:Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static THEME:Lazy<Mutex<HTHEME>> = Lazy::new(|| Mutex::new(HTHEME(0)));
static HWNDS:Lazy<Mutex<Vec<HWND>>> = Lazy::new(|| Mutex::new( Vec::new()));
static COUNTER: Counter = Counter::new_with_start(100);

struct ColorSchema {
    color:u32,
    border:u32,
    disabled:u32,
}

const DARK_COLOR_SCHEMA:ColorSchema = ColorSchema {
    color:0x00e7e0e0,
    border:0x00565659,
    disabled:0x00565659,
};

#[allow(dead_code)]
const LIGHT_COLOR_SCHEMA:ColorSchema = ColorSchema {
    color:0xc7bfbf99,
    border:0x00565659,
    disabled:0x00565659,
};

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq, Eq)]
struct X_MENU_TYPE(pub i32);
const XMT_STRING:X_MENU_TYPE = X_MENU_TYPE(0);
const XMT_CHECKBOX:X_MENU_TYPE = X_MENU_TYPE(1);
const XMT_RADIO:X_MENU_TYPE = X_MENU_TYPE(2);
const XMT_SUBMENU:X_MENU_TYPE = X_MENU_TYPE(3);
const XMT_SEPARATOR:X_MENU_TYPE = X_MENU_TYPE(4);
const XMT_DUMMY:X_MENU_TYPE = X_MENU_TYPE(5);
const MENU_SUBCLASS_ID: usize = 200;

#[derive(Clone)]
pub struct Menu {
    pub hwnd:HWND,
    menu:HMENU,
    menus:Vec<HMENU>,
    mode:i32,
}

#[allow(dead_code)]
#[derive(Clone)]
struct MenuItemData{
    id:Vec<u16>,
    label:Vec<u16>,
    value:Vec<u16>,
    menu_type:X_MENU_TYPE,
    name:Option<Vec<u16>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MenuEvent {
    pub id:String,
    pub value:String,
}

struct MenuMargin {
    menu:HMENU,
    index:u32,
}

impl Menu {
    pub fn new(hwnd:HWND) -> Self {

        let menu = unsafe { CreatePopupMenu().unwrap() };

        Self {
            hwnd,
            menu,
            menus:Vec::new(),
            mode:Theme::dark as _,
        }
    }

    pub fn new_with_theme(hwnd:HWND, theme:Theme) -> Self {

        let menu = unsafe { CreatePopupMenu().unwrap() };

        let mode = theme as i32;
        *MODE.lock().unwrap() = mode;
        draw_background_color(hwnd, menu).unwrap();

        Self {
            hwnd,
            menu,
            menus:Vec::new(),
            mode,
        }
    }

    pub fn popup_at(&self, x:i32, y:i32) -> Option<MenuEvent> {

        let id = unsafe { TrackPopupMenu(self.menu, TPM_TOPALIGN | TPM_LEFTALIGN | TPM_RETURNCMD, x, y, 0, self.hwnd, None) };

        if id.0 > 0 {
            let menu = Self::find_menu(self, id.0 as u32);
            return on_command(menu, id.0 as u32);
        }

        None
    }

    fn find_menu(&self, id:u32) -> HMENU {

        let mut info = MENUITEMINFOW::default();
        info.cbSize = size_of::<MENUITEMINFOW>() as u32;

        for menu in self.menus.clone() {
            unsafe {
                if GetMenuItemInfoW(menu, id, false, &mut info).is_ok() {
                    return menu;
                }
            }
        }

        self.menu
    }

    pub fn build(&self) -> Result<(), windows_core::Error> {
        draw_background_color(self.hwnd, self.menu)?;

        let mut menus = vec![self.menu];
        menus.extend(self.menus.clone());

        for menu in menus {
            Self::add_margin(self, menu, 0);
            let count = unsafe { GetMenuItemCount(menu) };
            Self::add_margin(self, menu, count as u32);
        }

        Ok(())
    }

    pub fn build_and_attach(&self) -> Result<(), windows_core::Error> {
        Self::build(self)?;
        Self::attach_menu_subclass_for_hwnd(self);
        Ok(())
    }

    pub fn text(&self, id:&str, label:&str) -> &Self {
        Self::_menu(self, id, label, "", None, None, XMT_STRING, None).unwrap();
        self
    }

    pub fn check(&self, id:&str, label:&str, value:&str, checked:bool) -> &Self {
        Self::_menu(self, id, label, value, None, Some(checked), XMT_CHECKBOX, None).unwrap();
        self
    }

    pub fn radio(&self, id:&str, label:&str, value:&str, name:&str, checked:bool) -> &Self {
        Self::_menu(self, id, label, value, Some(name), Some(checked), XMT_RADIO, None).unwrap();
        self
    }

    pub fn submenu(&mut self, label:&str) -> Self {
        let mii = Self::_menu(self, label, label, "", None, None, XMT_SUBMENU, None).unwrap();
        self.menus.push(mii.hSubMenu);

        Menu {
            hwnd:self.hwnd,
            menu: mii.hSubMenu,
            menus:Vec::new(),
            mode:self.mode,
        }
    }

    pub fn separator(&self) -> &Self {
        Self::_menu(self, "", "", "", None, None, XMT_SEPARATOR, None).unwrap();
        self
    }

    fn add_margin(&self, menu:HMENU, index:u32){
        Self::_menu(self, "", "", "", None, None, XMT_DUMMY, Some(MenuMargin{menu, index})).unwrap();
    }

    fn _menu(&self, id:&str, label:&str, value:&str, name:Option<&str>, checked:Option<bool>, menu_type:X_MENU_TYPE, margin:Option<MenuMargin>) -> Result<MENUITEMINFOW, windows_core::Error> {

        let mut mii = MENUITEMINFOW::default();
        mii.cbSize = size_of::<MENUITEMINFOW>() as u32;
        mii.fMask = MIIM_FTYPE | MIIM_ID | MIIM_DATA;
        mii.fType = MFT_OWNERDRAW;

        match menu_type {
            XMT_STRING => {
                mii.fType |= MFT_STRING;
            },

            XMT_CHECKBOX | XMT_RADIO => {
                if checked.unwrap() {
                    mii.fMask |= MIIM_STATE;
                    mii.fState = MFS_CHECKED;
                }
            },

            XMT_SUBMENU => {
                let submenu = unsafe { CreatePopupMenu()? };
                mii.fMask |= MIIM_SUBMENU;
                mii.hSubMenu = submenu;
            },

            XMT_SEPARATOR | XMT_DUMMY => {
                mii.fType |= MFT_SEPARATOR;
            },
            _ => {}
        }
        if menu_type == XMT_CHECKBOX {
            mii.fState = MFS_CHECKED;
        }else{
            mii.fType |= MFT_STRING;
        }
        mii.wID = COUNTER.next();
        mii.cch = label.len() as u32;
        let mut lpstr_text = encode_wide(label);
        mii.dwTypeData = PWSTR::from_raw(lpstr_text.as_mut_ptr());
        // Including null terminator
        mii.cch += 1;

        let name = if menu_type == XMT_RADIO { Some(encode_wide(name.unwrap()))} else { None };

        let itemdata = MenuItemData{
            id:encode_wide(id),
            label:encode_wide(label),
            value:encode_wide(value),
            menu_type,
            name
        };
        mii.dwItemData = Box::into_raw(Box::new(itemdata)) as usize;

        if margin.is_none() {
            let count = unsafe { GetMenuItemCount(self.menu) };
            unsafe { InsertMenuItemW(self.menu, count as u32, TRUE, &mii)? };
        } else {
            let margin = margin.unwrap();
            unsafe { InsertMenuItemW(margin.menu, margin.index, TRUE, &mii)? };
        }

        Ok(mii)
    }

    fn attach_menu_subclass_for_hwnd(&self) {
        unsafe {
            SetWindowSubclass(
                self.hwnd,
                Some(menu_subclass_proc),
                MENU_SUBCLASS_ID,
                0,
            );
        }
    }
}

fn open_theme(hwnd:HWND) -> HTHEME{
    if *HAS_THEME.lock().unwrap() {
        *THEME.lock().unwrap()
    } else {
        let theme = unsafe { OpenThemeDataEx(hwnd, windows_core::w!("Menu"), OTD_NONCLIENT) };
        *THEME.lock().unwrap() = theme;
        *HAS_THEME.lock().unwrap() = true;
        theme
    }
}

fn close_theme(){
    if *HAS_THEME.lock().unwrap() {
        unsafe { CloseThemeData(*THEME.lock().unwrap()).unwrap() };
        *THEME.lock().unwrap() = HTHEME(0);
        *HAS_THEME.lock().unwrap() = false;
    }
}

#[allow(unused_variables)]
unsafe extern "system" fn menu_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    uidsubclass: usize,
    dwrefdata: usize,
) -> LRESULT {

    match msg {

        WM_DESTROY => {
            let mut hwnds = HWNDS.lock().unwrap();
            if (*hwnds).len() > 0 {
                (*hwnds).pop().unwrap();
            }
            if (*hwnds).is_empty() {
                close_theme();
            }
            RemoveWindowSubclass(hwnd, Some(menu_subclass_proc), MENU_SUBCLASS_ID);
            DefSubclassProc(hwnd, msg, wparam, lparam)
        }

        WM_MEASUREITEM => {
            MeasureItem(hwnd, std::mem::transmute::<isize, &mut MEASUREITEMSTRUCT>(lparam.0));
            LRESULT(0)
        }

        WM_DRAWITEM => {
            let theme = open_theme(hwnd);
            DrawItemFlat(theme, std::mem::transmute::<isize, &DRAWITEMSTRUCT>(lparam.0)).unwrap();
            LRESULT(0)
        }

        WM_INITMENUPOPUP => {
            draw_background_color(hwnd, HMENU(wparam.0 as isize)).unwrap();
            DefSubclassProc(hwnd, msg, wparam, lparam)
        }

        WM_APPTHEMECHANGE => {
            close_theme();
            open_theme(hwnd);
            *MODE.lock().unwrap() = wparam.0 as i32;
            DefSubclassProc(hwnd, msg, wparam, lparam)
        }

        _ => {
            DefSubclassProc(hwnd, msg, wparam, lparam)
        }
    }
}

#[allow(non_upper_case_globals)]
const s_kcxGap:i32 = 1;
#[allow(non_upper_case_globals)]
const s_kcxTextMargin:i32 = 2;
#[allow(non_upper_case_globals)]
const s_kcxButtonMargin:i32 = 3;
#[allow(non_upper_case_globals)]
const s_kcyButtonMargin:i32 = 3;

fn from_usize<'a, T>(data:usize) -> &'a T {
    let pmd_ptr = data as *const T;
    unsafe { &*pmd_ptr }
}

fn is_dark() -> bool {
    *MODE.lock().unwrap() == Theme::dark as i32
}

fn on_command(hmenu:HMENU, id:u32) -> Option<MenuEvent> {
    unsafe {
        let mut info = MENUITEMINFOW::default();
        info.cbSize = size_of::<MENUITEMINFOW>() as u32;
        info.fMask = MIIM_DATA | MIIM_STATE;
        if GetMenuItemInfoW(hmenu, id, false, &mut info).is_ok() {
            let item_data = info.dwItemData;
            let data = from_usize::<MenuItemData>(item_data);

            if data.menu_type == XMT_CHECKBOX {
                let checked = info.fState.0 & MFS_CHECKED.0 != 0;
                toggle_checked(hmenu, &mut info, id, false, !checked);
            }

            if data.menu_type == XMT_RADIO {
                let name = decode_wide(&data.name.as_ref().unwrap());
                toggle_radio(hmenu, id, &mut info, name);
            }

            let e = MenuEvent {
                id:decode_wide(&data.id),
                value:decode_wide(&data.value),
            };
            return Some(e);
        }

        None
    }
}

fn toggle_checked(hmenu:HMENU, info:&mut MENUITEMINFOW, item:u32, byposition:bool, check:bool){
    if check {
        info.fState = MFS_CHECKED;
    } else {
        info.fState = MFS_UNCHECKED;
    }
    unsafe { SetMenuItemInfoW(hmenu, item, byposition, info).unwrap() };
}

fn toggle_radio(hmenu:HMENU, selected_id:u32, selected_item_info:&mut MENUITEMINFOW, name:String){
    unsafe {
        let count = GetMenuItemCount(hmenu);
        let mut items = Vec::new();

        toggle_checked(hmenu, selected_item_info, selected_id, false, true);

        for i in 0..count {
            let mut info = MENUITEMINFOW::default();
            info.cbSize = size_of::<MENUITEMINFOW>() as u32;
            info.fMask = MIIM_DATA | MIIM_STATE | MIIM_ID;
            GetMenuItemInfoW(hmenu, i as u32, true, &mut info).unwrap();
            let itemdata = info.dwItemData;
            let data = from_usize::<MenuItemData>(itemdata);

            if info.wID != selected_id && data.menu_type == XMT_RADIO && decode_wide(data.name.as_ref().unwrap()) == name {
                items.push(info);
            }
        }

        items.iter_mut().enumerate().for_each(|(_, info)| {
            toggle_checked(hmenu, info, info.wID, false, false);
        });

    }
}

fn draw_background_color(hwnd:HWND, menu:HMENU) -> Result<(), windows_core::Error> {
    unsafe{

        let mut info = MENUINFO::default();
        info.cbSize = size_of::<MENUINFO>() as u32;
        info.fMask = MIM_BACKGROUND | MIM_APPLYTOSUBMENUS | MIM_MENUDATA;
        GetMenuInfo(menu, &mut info)?;

        let mode = info.dwMenuData as i32;
        if mode != *MODE.lock().unwrap() {

            let theme = open_theme(hwnd);
            let bitmap = GetThemeBitmap(theme, MENU_POPUPBACKGROUND.0, 0, TMT_DIBDATA, GBF_DIRECT)?;
            let dc = CreateCompatibleDC(None);
            let old = SelectObject(dc, bitmap);
            let theme_color = GetPixel(dc,0,0);
            SelectObject(dc,old);
            ReleaseDC(hwnd, dc);

            info.hbrBack = CreateSolidBrush(theme_color);
            info.dwMenuData = *MODE.lock().unwrap() as _;
            SetMenuInfo(menu, &info)?;
        }
    }

    Ok(())
}

#[allow(non_snake_case)]
fn MeasureItem(hWnd:HWND, lpMeasureItemStruct:&mut MEASUREITEMSTRUCT){

    unsafe {

        let info = GetSystemSettings(hWnd);
        let pmd_ptr = lpMeasureItemStruct.itemData as *const MenuItemData;
        let pmd = &*pmd_ptr;

        match pmd.menu_type {

            XMT_SEPARATOR => {
            // separator - use half system height and zero width
            lpMeasureItemStruct.itemHeight = (GetSystemMetrics(SM_CYMENU) as u32 + 4u32) / 2u32;
            lpMeasureItemStruct.itemWidth  = 0;
            },

            XMT_DUMMY => {
                lpMeasureItemStruct.itemHeight = 2;
                lpMeasureItemStruct.itemWidth  = 0;
            },

            _ => {
                // compute size of text - use DrawText with DT_CALCRECT
                let dc:HDC = GetDC(hWnd);
                let hOldFont:HGDIOBJ = SelectObject(dc, info.m_fontMenu);
                let mut rcText = RECT::default();

                let mut text = pmd.label.clone();
                DrawTextW(dc, text.as_mut_slice(), &mut rcText, DT_SINGLELINE | DT_LEFT | DT_VCENTER | DT_CALCRECT);

                let mut cx = rcText.right - rcText.left;
                SelectObject(dc, hOldFont);

                let mut lf = LOGFONTW::default();
                GetObjectW(info.m_fontMenu, size_of::<LOGFONTW>() as i32, Some(&mut lf as *mut _ as *mut c_void));

                let mut cy = lf.lfHeight;
                if cy < 0 {
                    cy = -cy;
                }
                let cyMargin = 8;
                cy += cyMargin;

                let mut m_szBitmap = SIZE::default();
                m_szBitmap.cx = 16;
                m_szBitmap.cy = 15;
                let mut m_szButton = SIZE::default();
                m_szButton.cx = m_szBitmap.cx + 2 * s_kcxButtonMargin;
                m_szButton.cy = m_szBitmap.cy + 2 * s_kcyButtonMargin;

                // height of item is the bigger of these two
                lpMeasureItemStruct.itemHeight = std::cmp::max(cy as u32 + 4u32, m_szButton.cy as u32);

                // width is width of text plus a bunch of stuff
                cx += 2 * s_kcxTextMargin;   // L/R margin for readability
                cx += s_kcxGap;              // space between button and menu text
                cx += 2 * m_szButton.cx;     // button width (L=button; R=empty margin)
                cx += info.m_cxExtraSpacing;      // extra between item text and accelerator keys

                // Windows adds 1 to returned value
                cx -= GetSystemMetrics(SM_CXMENUCHECK) - 1;
                // done deal
                lpMeasureItemStruct.itemWidth = (cx + 10) as u32;

                ReleaseDC(hWnd, dc);
            }
        }
    }
}

#[allow(non_snake_case)]
fn DrawItemFlat(theme:HTHEME, lpDrawItemStruct:&DRAWITEMSTRUCT) -> Result<(), windows_core::Error> {

    unsafe {

        let mut m_szBitmap = SIZE::default();
        m_szBitmap.cx = 16;
        m_szBitmap.cy = 15;
        let mut m_szButton = SIZE::default();
        m_szButton.cx = m_szBitmap.cx + 2 * s_kcxButtonMargin;
        m_szButton.cy = m_szBitmap.cy + 2 * s_kcyButtonMargin;

        let pmd_ptr = lpDrawItemStruct.itemData as *const MenuItemData;
        let pmd = &*pmd_ptr;

        let dc = lpDrawItemStruct.hDC;
        let mut rcItem = lpDrawItemStruct.rcItem;

        let _bDisabled = (lpDrawItemStruct.itemState.0 & ODS_GRAYED.0) != 0;
        let bSelected = (lpDrawItemStruct.itemState.0 & ODS_SELECTED.0) != 0;
        let bChecked = (lpDrawItemStruct.itemState.0 & ODS_CHECKED.0) != 0;

        DrawThemeBackgroundEx(theme, dc, MENU_POPUPGUTTER.0, 0, &mut rcItem, None)?;

        // paint background
        if bSelected || (lpDrawItemStruct.itemAction == ODA_SELECT)
        {
          if bSelected {
            DrawThemeBackgroundEx(theme, dc, MENU_POPUPITEM.0, MPI_HOT.0, &mut rcItem, None)?;
          }else{
            DrawThemeBackgroundEx(theme, dc, MENU_POPUPITEM.0, MPI_NORMAL.0, &mut rcItem, None)?;
          }
        }

        match pmd.menu_type {
            XMT_SEPARATOR => {
                draw_separator(dc, rcItem)?;
            },

            XMT_DUMMY => {
                DrawThemeBackgroundEx(theme, dc, MENU_POPUPITEM.0, MPI_NORMAL.0, &mut rcItem, None)?;
            },

            _ => {

                // button rect
                let mut rcButn = RECT{ left: rcItem.left, top:rcItem.top, right:rcItem.left + m_szButton.cx, bottom:rcItem.top + m_szButton.cy };
                // center vertically
                OffsetRect(&mut rcButn, 0, ((rcItem.bottom - rcItem.top) - (rcButn.bottom - rcButn.top)) / 2);

                // draw background and border for checked items
                if bChecked
                {
                    let mut rcCheck = rcButn.clone();
                    InflateRect(&mut rcCheck as *mut _ as *mut RECT, -1, -1);
                    DrawThemeBackgroundEx(theme, dc, MENU_POPUPCHECK.0, MC_CHECKMARKNORMAL.0, &mut rcCheck, None)?;
                }

                // draw item text
                let cxButn = m_szButton.cx;
                // calc text rectangle and colors
                let mut rcText = rcItem.clone();
                rcText.left += cxButn + s_kcxGap + s_kcxTextMargin;
                rcText.right -= cxButn;

                let mut textRect = rcText.clone();

                if pmd.menu_type == XMT_SUBMENU {
                    let mut arrowR  = rcItem.clone();
                    let arrowSize = GetSystemMetrics(SM_CXHSCROLL);
                    textRect.right -= arrowSize;
                    arrowR.left = rcItem.right - arrowSize;

                    // center vertically
                    OffsetRect(&mut arrowR as *mut _ as *mut RECT, 0, ((rcItem.bottom - rcItem.top) - (arrowR.bottom - arrowR.top)) / 2);
                    DrawThemeBackgroundEx(theme, dc, MENU_POPUPSUBMENU.0, MSM_NORMAL.0, &mut arrowR, None)?;

                }

                DrawMenuText(dc, &textRect, PCWSTR::from_raw(pmd.label.as_ptr()))?;

                ExcludeClipRect(dc, rcItem.left, rcItem.top, rcItem.right, rcItem.bottom);

            }
        }

    }

    Ok(())
}

#[allow(non_snake_case)]
fn draw_separator(dc:HDC, rcItem:RECT) -> Result<(), windows_core::Error> {
    unsafe {

        if is_dark() {

            let mut rc = rcItem.clone();
            rc.top += (rcItem.bottom - rcItem.top) / 2;

            let hPen:HPEN = CreatePen(PS_SOLID, 1, COLORREF(DARK_COLOR_SCHEMA.border));
            let hFontOld:HGDIOBJ = SelectObject(dc,hPen);
            MoveToEx(dc, rc.left, rc.top, None);
            LineTo(dc, rc.right, rc.top);
            SelectObject(dc,hFontOld);

        } else {

            let mut rc = rcItem.clone();
            let size = (rc.bottom - rc.top) / 2;
            rc.bottom -= size / 2;
            rc.top += size / 2;
            DrawThemeBackgroundEx(*THEME.lock().unwrap(), dc, MENU_POPUPSEPARATOR.0, 0, &mut rc, None)?;

        }
    }

    Ok(())
}

#[allow(non_snake_case)]
fn DrawMenuText(dc:HDC, rc:&RECT, lpstrText:PCWSTR) -> Result<(), windows_core::Error>{
    unsafe {

        let text = lpstrText.to_string().unwrap();
        let nTab = text.contains("\t");
        let texts = text.split("\t").collect::<Vec::<&str>>();

        let mut rect = rc.clone();
        let mut first = encode_wide(texts[0]);
        let mut second = if nTab { encode_wide(texts[1]) } else { encode_wide("") };

        if is_dark() {
            let mut info:NONCLIENTMETRICSW = NONCLIENTMETRICSW::default();
            info.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
            SystemParametersInfoW(SPI_GETNONCLIENTMETRICS, size_of::<NONCLIENTMETRICSW>() as u32, Some(&mut info as *mut _ as *mut c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0))?;

            SetBkMode(dc, TRANSPARENT);
            SetTextColor(dc, COLORREF(DARK_COLOR_SCHEMA.color));

            let mut font = info.lfMenuFont;
            font.lfWeight = 700;

            let m_fontMenu:HFONT = CreateFontIndirectW(&font);
            let hFontOld:HGDIOBJ = SelectObject(dc,m_fontMenu);

            DrawTextW(dc, &mut first, &mut rect, DT_SINGLELINE | DT_LEFT | DT_VCENTER);

            SetTextColor(dc, COLORREF(DARK_COLOR_SCHEMA.disabled));
            DrawTextW(dc, &mut second, &mut rect, DT_SINGLELINE | DT_RIGHT | DT_VCENTER);

            SelectObject(dc,hFontOld);

        } else {
            DrawThemeTextEx(*THEME.lock().unwrap(), dc, MENU_POPUPITEM.0, MPI_NORMAL.0, first.as_mut(), DT_SINGLELINE | DT_LEFT | DT_VCENTER, &mut rect, None)?;
            DrawThemeTextEx(*THEME.lock().unwrap(), dc, MENU_POPUPITEM.0, MPI_NORMAL.0, second.as_mut(), DT_SINGLELINE | DT_RIGHT | DT_VCENTER, &mut rect, None)?;
        }
    }

    Ok(())
}

#[allow(non_snake_case)]
struct Info {
    m_fontMenu:HFONT,
    m_cxExtraSpacing:i32,
}

#[allow(non_snake_case)]
fn GetSystemSettings(hWnd: HWND) -> Info
{
    let mut sysinfo:Info = Info{m_fontMenu:HFONT::default(), m_cxExtraSpacing:10};
    unsafe {
        // refresh our font
        let mut info:NONCLIENTMETRICSW = NONCLIENTMETRICSW::default();
        info.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
        let bRet = SystemParametersInfoW(SPI_GETNONCLIENTMETRICS, size_of::<NONCLIENTMETRICSW>() as u32, Some(&mut info as *mut _ as *mut c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0));

        if bRet.is_ok()
        {
            let mut font = info.lfMenuFont;
            font.lfWeight = 700;

            let hFontMenu:HFONT = CreateFontIndirectW(&font);

            sysinfo.m_fontMenu = hFontMenu;
        }

        // check if we need extra spacing for menu item text
        let dc:HDC = GetDC(hWnd);
        let hFontOld:HGDIOBJ = SelectObject(dc,sysinfo.m_fontMenu);
        let mut rcText = RECT::default();

        DrawTextW(dc, encode_wide("\t").as_mut(), &mut rcText, DT_SINGLELINE | DT_LEFT | DT_VCENTER | DT_CALCRECT);
        if(rcText.right - rcText.left) < 4
        {
            SetRectEmpty(&mut rcText);
            DrawTextW(dc, encode_wide("x").as_mut(), &mut rcText, DT_SINGLELINE | DT_LEFT | DT_VCENTER | DT_CALCRECT);
            sysinfo.m_cxExtraSpacing = std::cmp::max(10, rcText.right - rcText.left);
        }
        SelectObject(dc,hFontOld);

        ReleaseDC(hWnd, dc);
    }

    sysinfo
}


