use serde::Serialize;
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, SIZE, TRUE, WPARAM};
use windows::Win32::Graphics::Gdi::{CreateCompatibleDC, CreateFontIndirectW, CreatePen, CreateSolidBrush, DrawTextW, ExcludeClipRect, GetDC, GetObjectW, GetPixel, InflateRect, LineTo, MoveToEx, OffsetRect, ReleaseDC, SelectObject, SetBkMode, SetRectEmpty, SetTextColor, DT_CALCRECT, DT_LEFT, DT_RIGHT, DT_SINGLELINE, DT_VCENTER, HDC, HFONT, HGDIOBJ, HPEN, LOGFONTW, PS_SOLID, TRANSPARENT};
use windows::Win32::UI::Controls::{CloseThemeData, DrawThemeBackgroundEx, DrawThemeTextEx, GetThemeBitmap, OpenThemeDataEx, DRAWITEMSTRUCT, GBF_DIRECT, HTHEME, MC_CHECKMARKNORMAL, MEASUREITEMSTRUCT, MENU_POPUPBACKGROUND, MENU_POPUPCHECK, MENU_POPUPGUTTER, MENU_POPUPITEM, MENU_POPUPSEPARATOR, MENU_POPUPSUBMENU, MPI_HOT, MPI_NORMAL, MSM_NORMAL, ODA_SELECT, ODS_CHECKED, ODS_GRAYED, ODS_SELECTED, OTD_NONCLIENT, TMT_DIBDATA};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{CreatePopupMenu, GetMenuInfo, GetMenuItemCount, GetMenuItemInfoW, GetSystemMetrics, InsertMenuItemW, SetMenuInfo, SetMenuItemInfoW, SystemParametersInfoW, TrackPopupMenu, HMENU, MENUINFO, MENUITEMINFOW, MFS_CHECKED, MFS_UNCHECKED, MFT_OWNERDRAW, MFT_SEPARATOR, MFT_STRING, MIIM_DATA, MIIM_FTYPE, MIIM_ID, MIIM_STATE, MIIM_SUBMENU, MIM_APPLYTOSUBMENUS, MIM_BACKGROUND, MIM_MENUDATA, NONCLIENTMETRICSW, SM_CXHSCROLL, SM_CXMENUCHECK, SM_CYMENU, SPI_GETNONCLIENTMETRICS, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, TPM_LEFTALIGN, TPM_RETURNCMD, TPM_TOPALIGN, WM_DESTROY, WM_DRAWITEM, WM_INITMENUPOPUP, WM_MEASUREITEM};
use windows_core::{PCWSTR, PWSTR};

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
const XMT_MARGIN:X_MENU_TYPE = X_MENU_TYPE(5);
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
        Self::_menu(self, "", "", "", None, None, XMT_MARGIN, Some(MenuMargin{menu, index})).unwrap();
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

            XMT_SEPARATOR | XMT_MARGIN => {
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
        mii.dwItemData = Box::into_raw(Box::new(itemdata)) as _;

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

unsafe extern "system" fn menu_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uidsubclass: usize,
    _dwrefdata: usize,
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
            measure_item(hwnd, std::mem::transmute::<isize, &mut MEASUREITEMSTRUCT>(lparam.0)).unwrap();
            LRESULT(0)
        }

        WM_DRAWITEM => {
            let theme = open_theme(hwnd);
            draw_item(theme, std::mem::transmute::<isize, &DRAWITEMSTRUCT>(lparam.0)).unwrap();
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

fn from_usize<'a, T>(data:usize) -> &'a T {
    let item_data_ptr = data as *const T;
    unsafe { &*item_data_ptr }
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

const V_MARGIN:i32 = 1;
const H_MARGIN:i32 = 2;
static BASE_SIZE:Lazy<SIZE> = Lazy::new(|| {
    let mut size = SIZE::default();
    size.cx = 16 + 2 * 3;
    size.cy = 15 + 2 * 3;
    size
});

fn measure_item(hwnd:HWND, measure_item_struct:&mut MEASUREITEMSTRUCT) -> Result<(), windows_core::Error> {

    unsafe {

        let item_data_ptr = measure_item_struct.itemData as *const MenuItemData;
        let item_data = &*item_data_ptr;

        match item_data.menu_type {

            XMT_SEPARATOR => {
                // separator - use half system height and zero width
                measure_item_struct.itemHeight = (GetSystemMetrics(SM_CYMENU) as u32 + 4u32) / 2u32;
                measure_item_struct.itemWidth  = 0;
            },

            XMT_MARGIN => {
                measure_item_struct.itemHeight = 2;
                measure_item_struct.itemWidth  = 0;
            },

            _ => {

                let dc:HDC = GetDC(hwnd);
                let menu_font = get_font()?;
                let font:HFONT = CreateFontIndirectW(&menu_font);
                let old_font:HGDIOBJ = SelectObject(dc, font);
                let mut text_rect = RECT::default();

                let mut text = item_data.label.clone();
                DrawTextW(dc, text.as_mut_slice(), &mut text_rect, DT_SINGLELINE | DT_LEFT | DT_VCENTER | DT_CALCRECT);

                let mut cx = text_rect.right - text_rect.left;
                SelectObject(dc, old_font);

                let mut log_font = LOGFONTW::default();
                GetObjectW(font, size_of::<LOGFONTW>() as i32, Some(&mut log_font as *mut _ as *mut c_void));

                let mut cy = log_font.lfHeight;
                if cy < 0 {
                    cy = -cy;
                }
                let cy_margin = 8;
                cy += cy_margin;

                // height of item is the bigger of these two
                measure_item_struct.itemHeight = std::cmp::max(cy as u32 + 4u32, BASE_SIZE.cy as u32);

                // L/R margin for readability
                cx += 2 * H_MARGIN;
                // space between button and menu text
                cx += V_MARGIN;
                // button width (L=button; R=empty margin)
                cx += 2 * BASE_SIZE.cx;
                // extra padding
                cx += 20;

                // Windows adds 1 to returned value
                cx -= GetSystemMetrics(SM_CXMENUCHECK) - 1;

                measure_item_struct.itemWidth = (cx + 10) as u32;

                ReleaseDC(hwnd, dc);
            }
        }
    }

    Ok(())
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

fn draw_item(theme:HTHEME, draw_item_struct:&DRAWITEMSTRUCT) -> Result<(), windows_core::Error> {

    unsafe {
        let item_data_ptr = draw_item_struct.itemData as *const MenuItemData;
        let item_data = &*item_data_ptr;

        let dc = draw_item_struct.hDC;
        let mut item_rect = draw_item_struct.rcItem;

        let _disabled = (draw_item_struct.itemState.0 & ODS_GRAYED.0) != 0;
        let selected = (draw_item_struct.itemState.0 & ODS_SELECTED.0) != 0;
        let checked = (draw_item_struct.itemState.0 & ODS_CHECKED.0) != 0;

        DrawThemeBackgroundEx(theme, dc, MENU_POPUPGUTTER.0, 0, &mut item_rect, None)?;

        // paint background
        if selected || (draw_item_struct.itemAction == ODA_SELECT){
            if selected {
                DrawThemeBackgroundEx(theme, dc, MENU_POPUPITEM.0, MPI_HOT.0, &mut item_rect, None)?;
            }else{
                DrawThemeBackgroundEx(theme, dc, MENU_POPUPITEM.0, MPI_NORMAL.0, &mut item_rect, None)?;
            }
        }

        match item_data.menu_type {
            XMT_SEPARATOR => {
                draw_separator(dc, item_rect)?;
            },

            XMT_MARGIN => {
                DrawThemeBackgroundEx(theme, dc, MENU_POPUPITEM.0, MPI_NORMAL.0, &mut item_rect, None)?;
            },

            _ => {

                if checked{
                    // button rect
                    let mut rect = RECT{ left: item_rect.left, top:item_rect.top, right:item_rect.left + BASE_SIZE.cx, bottom:item_rect.top + BASE_SIZE.cy };
                    // center vertically
                    OffsetRect(&mut rect, 0, ((item_rect.bottom - item_rect.top) - (rect.bottom - rect.top)) / 2);
                    let mut check_rect = rect.clone();
                    InflateRect(&mut check_rect as *mut _ as *mut RECT, -1, -1);
                    DrawThemeBackgroundEx(theme, dc, MENU_POPUPCHECK.0, MC_CHECKMARKNORMAL.0, &mut check_rect, None)?;
                }

                let mut text_rect = item_rect.clone();
                text_rect.left += BASE_SIZE.cx + V_MARGIN + H_MARGIN;
                text_rect.right -= BASE_SIZE.cx;

                if item_data.menu_type == XMT_SUBMENU {
                    let mut arrow_rect  = item_rect.clone();
                    let arrow_size = GetSystemMetrics(SM_CXHSCROLL);
                    text_rect.right -= arrow_size;
                    arrow_rect.left = item_rect.right - arrow_size;

                    // center vertically
                    OffsetRect(&mut arrow_rect as *mut _ as *mut RECT, 0, ((item_rect.bottom - item_rect.top) - (arrow_rect.bottom - arrow_rect.top)) / 2);
                    DrawThemeBackgroundEx(theme, dc, MENU_POPUPSUBMENU.0, MSM_NORMAL.0, &mut arrow_rect, None)?;

                }

                draw_menu_text(dc, &text_rect, PCWSTR::from_raw(item_data.label.as_ptr()))?;
                ExcludeClipRect(dc, item_rect.left, item_rect.top, item_rect.right, item_rect.bottom);
            }
        }
    }

    Ok(())
}

fn draw_separator(dc:HDC, rect:RECT) -> Result<(), windows_core::Error> {
    unsafe {
        let mut separator_rect = rect.clone();

        if is_dark() {
            separator_rect.top += (rect.bottom - rect.top) / 2;

            let pen:HPEN = CreatePen(PS_SOLID, 1, COLORREF(DARK_COLOR_SCHEMA.border));
            let old_pen:HGDIOBJ = SelectObject(dc,pen);
            MoveToEx(dc, separator_rect.left, separator_rect.top, None);
            LineTo(dc, separator_rect.right, separator_rect.top);
            SelectObject(dc,old_pen);
        } else {
            let size = (separator_rect.bottom - separator_rect.top) / 2;
            separator_rect.bottom -= size / 2;
            separator_rect.top += size / 2;
            DrawThemeBackgroundEx(*THEME.lock().unwrap(), dc, MENU_POPUPSEPARATOR.0, 0, &mut separator_rect, None)?;
        }
    }

    Ok(())
}

fn draw_menu_text(dc:HDC, rect:&RECT, raw_text:PCWSTR) -> Result<(), windows_core::Error> {
    unsafe {

        let text = raw_text.to_string().unwrap();
        let has_tab = text.contains("\t");
        let texts = text.split("\t").collect::<Vec::<&str>>();

        let mut text_rect = rect.clone();
        let mut first = encode_wide(texts[0]);
        let mut second = if has_tab { encode_wide(texts[1]) } else { encode_wide("") };

        if is_dark() {

            SetBkMode(dc, TRANSPARENT);
            SetTextColor(dc, COLORREF(DARK_COLOR_SCHEMA.color));

            let menu_font = get_font()?;
            let font:HFONT = CreateFontIndirectW(&menu_font);
            let old_font:HGDIOBJ = SelectObject(dc,font);

            DrawTextW(dc, &mut first, &mut text_rect, DT_SINGLELINE | DT_LEFT | DT_VCENTER);

            SetTextColor(dc, COLORREF(DARK_COLOR_SCHEMA.disabled));
            DrawTextW(dc, &mut second, &mut text_rect, DT_SINGLELINE | DT_RIGHT | DT_VCENTER);

            SelectObject(dc,old_font);

        } else {
            DrawThemeTextEx(*THEME.lock().unwrap(), dc, MENU_POPUPITEM.0, MPI_NORMAL.0, first.as_mut(), DT_SINGLELINE | DT_LEFT | DT_VCENTER, &mut text_rect, None)?;
            DrawThemeTextEx(*THEME.lock().unwrap(), dc, MENU_POPUPITEM.0, MPI_NORMAL.0, second.as_mut(), DT_SINGLELINE | DT_RIGHT | DT_VCENTER, &mut text_rect, None)?;
        }
    }

    Ok(())
}

fn get_font() -> Result<LOGFONTW, windows_core::Error> {
    let mut info:NONCLIENTMETRICSW = NONCLIENTMETRICSW::default();
    info.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
    unsafe { SystemParametersInfoW(SPI_GETNONCLIENTMETRICS, size_of::<NONCLIENTMETRICSW>() as u32, Some(&mut info as *mut _ as *mut c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0))? };

    let mut menu_font = info.lfMenuFont;
    menu_font.lfWeight = 700;

    Ok(menu_font)
}

#[allow(non_snake_case)]
struct Info {
    m_fontMenu:HFONT,
    m_cxExtraSpacing:i32,
}

#[allow(non_snake_case)]
#[allow(dead_code)]
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


