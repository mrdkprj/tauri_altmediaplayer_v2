/*
use crate::settings::Settings;

pub fn create_player_menu(_window:&tauri::WebviewWindow, _settings:&Settings) -> tauri::Result<()> {
    Ok(())
}

pub fn create_playlist_menu(_window:&tauri::WebviewWindow, _settings:&Settings) -> tauri::Result<()> {

    Ok(())
}
*/
#[allow(unused_imports)]
use crate::menu;
#[allow(unused_imports)]
use crate::settings::Settings;
#[allow(unused_imports)]
use tauri::Manager;
#[allow(unused_imports)]
use webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_COMMAND;
#[allow(unused_imports)]
use webview2_com::{
    pwstr_from_str,string_from_pcwstr,CustomItemSelectedEventHandler,ContextMenuRequestedEventHandler,CreateCoreWebView2CompositionControllerCompletedHandler,
    Microsoft::Web::WebView2::Win32::{
        ICoreWebView2, ICoreWebView2ContextMenuRequestedEventArgs, ICoreWebView2_11, ICoreWebView2_2,ICoreWebView2_13, COREWEBVIEW2_PREFERRED_COLOR_SCHEME,
        ICoreWebView2Environment9,ICoreWebView2ContextMenuItem,ICoreWebView2ContextMenuItemCollection,ICoreWebView2Environment10,ICoreWebView2CompositionController,
        ICoreWebView2Environment,COREWEBVIEW2_MOUSE_EVENT_KIND_MOVE,COREWEBVIEW2_MOUSE_EVENT_VIRTUAL_KEYS_NONE
    }
};

#[allow(unused_imports)]
use windows::Win32::Foundation::*;
#[allow(unused_imports)]
use windows::Win32::Graphics::Gdi::*;
#[allow(unused_imports)]
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::UI::Controls::*;
#[allow(unused_imports)]
use windows::Win32::UI::Shell::*;
#[allow(unused_imports)]
use windows_core::{s, HSTRING, PCSTR};
#[allow(unused_imports)]
use windows_core::{ComInterface, PCWSTR, PWSTR, HRESULT};
#[allow(unused_imports)]
use windows::Win32::System::WinRT::EventRegistrationToken;
#[allow(unused_imports)]
use windows::Win32::UI::WindowsAndMessaging::*;
#[allow(unused_imports)]
use once_cell::sync::Lazy;
#[allow(unused_imports)]
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute,DWMWA_USE_IMMERSIVE_DARK_MODE};
#[allow(unused_imports)]
use std::collections::HashMap;
#[allow(unused_imports)]
use std::ffi::c_void;
#[allow(unused_imports)]
use std::ffi::OsString;
#[allow(unused_imports)]
use std::mem::size_of;
#[allow(unused_imports)]
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::sync::Mutex;

#[allow(dead_code)]
static CELL:Lazy<Mutex<HWND>> = Lazy::new(|| Mutex::new( HWND::default()));
static CELL2:Lazy<Mutex<HMENU>> = Lazy::new(|| Mutex::new( HMENU(0)));
static CELL3:Lazy<Mutex<HTHEME>> = Lazy::new(|| Mutex::new( HTHEME(0)));

pub fn create_player_menu(_window:&tauri::WebviewWindow, _settings:&Settings) -> tauri::Result<()> {



    // let window_ = window.clone();
    // menu::on_menu_event(Box::new( move |id| {
    //     window_.emit_to(tauri::EventTarget::WebviewWindow { label: id.label.clone() }, "contextmenu-event", id).unwrap();
    // }));

    // menu::create_player_menu(window, settings)
    menu::create_player_menu(_window, _settings)?;

    Ok(())
}

pub fn create_playlist_menu(_window:&tauri::WebviewWindow, _settings:&Settings) -> tauri::Result<()> {
    //menu::create_playlist_menu(_window, _settings)?;
    allow_dark_mode_for_app(true);
    creaet_test(_window).unwrap();
    Ok(())
}

#[allow(dead_code)]
struct ICoreWebView2Ext (ICoreWebView2);

impl ICoreWebView2Ext {
    pub fn cast<T:ComInterface>(&self) -> Result<T, windows_core::Error>{
        self.0.cast::<T>()
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq)]
struct X_MENU_TYPE(pub i32);
const XMT_STRING:X_MENU_TYPE = X_MENU_TYPE(0);
const XMT_CHECKBOX:X_MENU_TYPE = X_MENU_TYPE(1);
//const XMT_RADIO:X_MENU_TYPE = X_MENU_TYPE(2);
const XMT_SUBMENU:X_MENU_TYPE = X_MENU_TYPE(3);
const XMT_SEPARATOR:X_MENU_TYPE = X_MENU_TYPE(4);

#[allow(non_snake_case)]
#[derive(Clone)]
struct _MenuItemData{
    fType:X_MENU_TYPE,
    lpstrText:Vec<u16>,
    id:i32,
}

impl Default for _MenuItemData{
    fn default() -> Self {
        Self {
            fType:X_MENU_TYPE(0),
            lpstrText:Vec::new(),
            id:0,
        }
    }
}

#[allow(dead_code)]
fn creaet_test(window:&tauri::WebviewWindow) -> Result<(), windows_core::Error>{

    let hw = window.hwnd().unwrap();
    *CELL.lock().unwrap() = hw;

    let theme = unsafe { OpenThemeDataEx(hw, windows_core::w!("MENU"), OTD_NONCLIENT) };
    *CELL3.lock().unwrap() = theme;
  //  let mutrc_itemm = RECT::default()

    let menu:HMENU = unsafe { CreatePopupMenu()? };
    add_menu_itemat(100, "Playback Speed \tCtrl+X", XMT_STRING, menu);
    add_menu_itemat(200, "Seek Speed\tCtrl+X", XMT_CHECKBOX, menu);
    AddSeparatorItemAt(menu);
    add_menu_itemat(300, "Fit To Window Size\tCtrl+X", XMT_STRING, menu);

    let menu2:HMENU = unsafe { CreatePopupMenu()? };
    add_submenu(400, "Add Tag To Comment", menu2, menu);
    add_menu_itemat(500, "Subment", XMT_STRING, menu2);

    *CELL2.lock().unwrap() = menu;

    unsafe {
        let bitmap = GetThemeBitmap(theme, MENU_POPUPITEM.0, 0, TMT_DIBDATA, GBF_DIRECT)?;
        let dc = CreateCompatibleDC(None);
        let old = SelectObject(dc, bitmap);
        let theme_color = GetPixel(dc,0,0);
        SelectObject(dc,old);
        ReleaseDC(hw, dc);

        let mut info = MENUINFO::default();
        info.cbSize = size_of::<MENUINFO>() as u32;
        info.fMask = MIM_BACKGROUND | MIM_APPLYTOSUBMENUS;
        let b = CreateSolidBrush(theme_color);
        info.hbrBack = b;
        SetMenuInfo(menu, &info)?;
    }

    attach_menu_subclass_for_hwnd(hw);

    window.with_webview( move |webview| {
        unsafe{

            let core_webview = ICoreWebView2Ext(webview.controller().CoreWebView2().unwrap());

            let core_webview_13: ICoreWebView2_11 = core_webview.cast::<ICoreWebView2_11>().unwrap();

            let handler = ContextMenuRequestedEventHandler::create(Box::new(handler2));
            let mut token: EventRegistrationToken = EventRegistrationToken::default();
            core_webview_13.add_ContextMenuRequested(&handler, &mut token).unwrap();

        }
    }).unwrap();

    Ok(())
}


#[allow(dead_code)]
#[allow(non_snake_case)]
fn handler2(_sender: Option<ICoreWebView2>, args: Option<ICoreWebView2ContextMenuRequestedEventArgs>) -> Result<(), windows_core::Error>{
    if args.is_some() {
        unsafe {
            let args = args.unwrap();

            args.SetHandled(true)?;
            let mut location:POINT = POINT::default();
            args.Location(&mut location)?;

            let hwnd:HWND = *CELL.lock().unwrap();
            let menu = *CELL2.lock().unwrap();

            ClientToScreen(hwnd, &mut location);

            let id = TrackPopupMenu(menu, TPM_TOPALIGN | TPM_LEFTALIGN | TPM_RETURNCMD, location.x, location.y, 0, hwnd, None);

            if id.0 > 0 {
                println!("{}", id.0);
            }

        }
    }
    Ok(())
}

struct Menus {
    hwnd:HWND,
    menu:HMENU,
}


fn add_menu_itemat(id:i32, label:&str, menu_type:X_MENU_TYPE, menu:HMENU) {

    let mut mii = MENUITEMINFOW::default();
    mii.cbSize = size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_FTYPE | MIIM_ID | MIIM_DATA | MIIM_STATE;
    mii.fType = MFT_OWNERDRAW;
    if menu_type == XMT_CHECKBOX {
        mii.fState = MFS_CHECKED;
    }else{
        mii.fType |= MFT_STRING;
    }
    mii.wID = id as u32;
    mii.cch = label.len() as u32;
    let mut lpstr_text = encode_wide(label);
    mii.dwTypeData = PWSTR::from_raw(lpstr_text.as_mut_ptr());
    mii.cch += 1; // Including null terminator

    let mut itemdata = _MenuItemData::default();
    itemdata.fType = menu_type;
    itemdata.lpstrText = encode_wide(label);
    itemdata.id = id;
    mii.dwItemData = Box::into_raw(Box::new(itemdata)) as usize;

    unsafe { InsertMenuItemW(menu, 0, TRUE, &mii).unwrap() };
}

fn add_submenu(id:i32, label:&str, menu:HMENU, parent:HMENU){

    let mut mii = MENUITEMINFOW::default();
    mii.cbSize = size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_FTYPE | MIIM_ID | MIIM_DATA | MIIM_SUBMENU;
    mii.fType = MFT_OWNERDRAW;
    mii.hSubMenu = menu;
    mii.wID = id as u32;
    mii.cch = label.len() as u32;
    let mut lpstr_text = encode_wide(label);
    mii.dwTypeData = PWSTR::from_raw(lpstr_text.as_mut_ptr());
    mii.cch += 1; // Including null terminator

    let mut itemdata = _MenuItemData::default();
    itemdata.fType = XMT_SUBMENU;
    itemdata.lpstrText = encode_wide(label);
    mii.dwItemData = Box::into_raw(Box::new(itemdata)) as usize;

    unsafe { InsertMenuItemW(parent, 0, TRUE, &mii).unwrap() };
}

#[allow(non_snake_case)]
fn AddSeparatorItemAt(menu:HMENU) {
    let mut mii = MENUITEMINFOW::default();
    mii.cbSize = size_of::<MENUITEMINFOW>() as u32;
    mii.fMask = MIIM_FTYPE | MIIM_DATA;
    mii.fType = MFT_SEPARATOR | MFT_OWNERDRAW;

    let mut itemdata = _MenuItemData::default();
    itemdata.fType = XMT_SEPARATOR;
    itemdata.lpstrText = encode_wide("");
    itemdata.id = 0;
    mii.dwItemData = Box::into_raw(Box::new(itemdata)) as usize;

    unsafe { InsertMenuItemW(menu, 0, TRUE, &mii).unwrap() };
}

const MENU_SUBCLASS_ID: usize = 200;
// const SUBMENU_SUBCLASS_ID: usize = 201;
// const CONTEXT_MENU_SUBCLASS_ID: usize = 203;
// const CONTEXT_SUBMENU_SUBCLASS_ID: usize = 204;

pub fn attach_menu_subclass_for_hwnd(hwnd: HWND) {
    unsafe {
        SetWindowSubclass(
            hwnd,
            Some(menu_subclass_proc),
            MENU_SUBCLASS_ID,
            //Box::into_raw(Box::new()) as _,
            //Box::into_raw(Box::new(hwnd)) as _,
            0,
        );
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
          println!("destory");
            CloseThemeData(*CELL3.lock().unwrap()).unwrap();
            *CELL3.lock().unwrap() = HTHEME(0);
            RemoveWindowSubclass(hwnd, Some(menu_subclass_proc), MENU_SUBCLASS_ID);
            DefSubclassProc(hwnd, msg, wparam, lparam)
        }

        WM_MEASUREITEM => {
            MeasureItem(hwnd, std::mem::transmute::<isize, &mut MEASUREITEMSTRUCT>(lparam.0));
            LRESULT(0)
        }

        WM_DRAWITEM => {
            DrawItemFlat(hwnd, std::mem::transmute::<isize, &DRAWITEMSTRUCT>(lparam.0));
            LRESULT(0)
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

#[allow(non_snake_case)]
fn MeasureItem(hWnd:HWND, lpMeasureItemStruct:&mut MEASUREITEMSTRUCT)
{

    // todo: if top or last, add margin
    unsafe {

        let info = GetSystemSettings(hWnd);
        let pmd_ptr = lpMeasureItemStruct.itemData as *const _MenuItemData;
        let pmd = &*pmd_ptr;

        println!("MeasureItem: {:?}", HSTRING::from_wide(&pmd.lpstrText).unwrap().to_string());

        if pmd.fType == XMT_SEPARATOR   // separator - use half system height and zero width
        {
            lpMeasureItemStruct.itemHeight = (GetSystemMetrics(SM_CYMENU) as u32 + 4u32) / 2u32;
            lpMeasureItemStruct.itemWidth  = 0;
        }
        else
        {
            // compute size of text - use DrawText with DT_CALCRECT
            let dc:HDC = GetDC(hWnd);
            let hOldFont:HGDIOBJ = SelectObject(dc, info.m_fontMenu);
            let mut rcText = RECT::default();

            let mut text = pmd.lpstrText.clone();
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

const DARK_TEXT_COLOR:u32 = 0x00e7e0e0;
const DARK_HIGHLITE_COLOR:u32 = 0x00565659;
// const DARK_BK_COLOR:u32 = 0x00252526;
#[allow(non_snake_case)]
fn DrawItemFlat(_hwnd:HWND, lpDrawItemStruct:&DRAWITEMSTRUCT){

    unsafe {

        let mut m_szBitmap = SIZE::default();
        m_szBitmap.cx = 16;
        m_szBitmap.cy = 15;
        let mut m_szButton = SIZE::default();
        m_szButton.cx = m_szBitmap.cx + 2 * s_kcxButtonMargin;
        m_szButton.cy = m_szBitmap.cy + 2 * s_kcyButtonMargin;

        let pmd_ptr = lpDrawItemStruct.itemData as *const _MenuItemData;
        let pmd = &*pmd_ptr;

        println!("DrawItemFlat: {:?}", lpDrawItemStruct.itemID);
        let hMenuWnd = WindowFromDC(lpDrawItemStruct.hDC);
        println!("hMenuWnd: {:?}", hMenuWnd);

        let dc = lpDrawItemStruct.hDC;
        let mut rcItem = lpDrawItemStruct.rcItem;

        let bDisabled = (lpDrawItemStruct.itemState.0 & ODS_GRAYED.0) != 0;
        let mut bSelected = (lpDrawItemStruct.itemState.0 & ODS_SELECTED.0) != 0;
        let bChecked = (lpDrawItemStruct.itemState.0 & ODS_CHECKED.0) != 0;

        DrawThemeBackgroundEx(*CELL3.lock().unwrap(), dc, MENU_POPUPGUTTER.0, 0, &mut rcItem, None).unwrap();

        // paint background
        if bSelected || (lpDrawItemStruct.itemAction == ODA_SELECT)
        {
          if bSelected {
            DrawThemeBackgroundEx(*CELL3.lock().unwrap(), dc, MENU_POPUPITEM.0, MPI_HOT.0, &mut rcItem, None).unwrap();
          }else{
            DrawThemeBackgroundEx(*CELL3.lock().unwrap(), dc, MENU_POPUPITEM.0, MPI_NORMAL.0, &mut rcItem, None).unwrap();
          }
        }

        if pmd.fType == XMT_SEPARATOR
        {
            // draw separator
            draw_separator(dc, rcItem, COLORREF(DARK_HIGHLITE_COLOR));
        }
        else		// not a separator
        {
            if lpDrawItemStruct.itemID as i32 == -1 {
                bSelected = false;
            }

            let mut rcButn = RECT{ left: rcItem.left, top:rcItem.top, right:rcItem.left + m_szButton.cx, bottom:rcItem.top + m_szButton.cy };   // button rect
            OffsetRect(&mut rcButn, 0, ((rcItem.bottom - rcItem.top) - (rcButn.bottom - rcButn.top)) / 2);          // center vertically

            // draw background and border for checked items
            if bChecked
            {
                let mut rcCheck = rcButn.clone();
                InflateRect(&mut rcCheck as *mut _ as *mut RECT, -1, -1);
                DrawThemeBackgroundEx(*CELL3.lock().unwrap(), dc, MENU_POPUPCHECK.0, MC_CHECKMARKNORMAL.0, &mut rcCheck, None).unwrap();
            }

            // draw item text
            let cxButn = m_szButton.cx;
            // calc text rectangle and colors
            let mut rcText = rcItem.clone();
            rcText.left += cxButn + s_kcxGap + s_kcxTextMargin;
            rcText.right -= cxButn;

            let color = if bDisabled {
                if bSelected {
                    GetSysColor(COLOR_GRAYTEXT)
                } else {
                    GetSysColor(COLOR_3DSHADOW)
                }
            } else {
                if bSelected {
                    DARK_TEXT_COLOR
                } else {
                    DARK_TEXT_COLOR
                }
            };

            let colorText = COLORREF(color);

            let mut textRect = rcText.clone();

            if pmd.fType == XMT_SUBMENU {
                let mut arrowR  = rcItem.clone();
                let arrowSize = GetSystemMetrics(SM_CXHSCROLL);
                textRect.right -= arrowSize;
                arrowR.left = rcItem.right - arrowSize;

                // center vertically
                OffsetRect(&mut arrowR as *mut _ as *mut RECT, 0, ((rcItem.bottom - rcItem.top) - (arrowR.bottom - arrowR.top)) / 2);
                DrawThemeBackgroundEx(*CELL3.lock().unwrap(), dc, MENU_POPUPSUBMENU.0, MSM_NORMAL.0, &mut arrowR, None).unwrap();

            }

            DrawMenuText(dc, &textRect, PCWSTR::from_raw(pmd.lpstrText.as_ptr()), colorText); // finally!

            ExcludeClipRect(dc, rcItem.left, rcItem.top, rcItem.right, rcItem.bottom);

        }

    }
}

#[allow(non_snake_case)]
fn draw_separator(dc:HDC, rcItem:RECT, color:COLORREF){
    unsafe {
        let mut rc = rcItem.clone();
        rc.top += (rc.bottom - rc.top) / 2;      // vertical center
        let hPen:HPEN = CreatePen(PS_SOLID, 1, color);
        let hFontOld:HGDIOBJ = SelectObject(dc,hPen);
        MoveToEx(dc, rc.left, rc.top, None);
        LineTo(dc, rc.right, rc.top);
        SelectObject(dc,hFontOld);
        //DrawThemeBackgroundEx(*CELL3.lock().unwrap(), dc, MENU_POPUPSEPARATOR.0, 0, &mut rc as *const _ as _, None).unwrap();
    }
}

#[allow(non_snake_case)]
fn DrawMenuText(dc:HDC, rc:&RECT, lpstrText:PCWSTR, color:COLORREF){
    unsafe {

        let mut info:NONCLIENTMETRICSW = NONCLIENTMETRICSW::default();
        info.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
        let bRet = SystemParametersInfoW(SPI_GETNONCLIENTMETRICS, size_of::<NONCLIENTMETRICSW>() as u32, Some(&mut info as *mut _ as *mut c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0));

        match bRet{
            Ok(_) => {
                SetBkMode(dc, TRANSPARENT);
                SetTextColor(dc, color);

                let mut font = info.lfMenuFont;
                font.lfWeight = 700;

                let m_fontMenu:HFONT = CreateFontIndirectW(&font);
                let hFontOld:HGDIOBJ = SelectObject(dc,m_fontMenu);

                let text = lpstrText.to_string().unwrap();
                let nTab = text.contains("\t");
                let texts = text.split("\t").collect::<Vec::<&str>>();

                let mut rect = rc.clone();
                let mut first = encode_wide(texts[0]);
                DrawTextW(dc, &mut first, &mut rect, DT_SINGLELINE | DT_LEFT | DT_VCENTER);
                //DrawThemeTextEx(*CELL3.lock().unwrap(), dc, MENU_POPUPITEM.0, MPI_NORMAL.0, first.as_mut(), DT_SINGLELINE | DT_LEFT | DT_VCENTER, &mut rect, None).unwrap();

                if nTab {
                    let mut second = encode_wide(texts[1]);
                    DrawTextW(dc, &mut second, &mut rect, DT_SINGLELINE | DT_RIGHT | DT_VCENTER);
                    //DrawThemeTextEx(*CELL3.lock().unwrap(), dc, MENU_POPUPITEM.0, MPI_NORMAL.0, second.as_mut(), DT_SINGLELINE | DT_RIGHT | DT_VCENTER, &mut rect, None).unwrap();
                }
                SelectObject(dc,hFontOld);
            },
            Err(e) => println!("SystemParametersInfoW failed: {:?}", e)
        }
    }
}

#[allow(dead_code)]
fn to_pcwstr(str:&str) -> PCWSTR {
    PCWSTR::from_raw(pwstr_from_str(str).as_ptr())
}
#[allow(dead_code)]
fn to_vec_u16(str:PWSTR) -> Vec::<u16>{
    unsafe {
        let rws = str.to_hstring().unwrap();
        rws.to_owned().as_wide().to_vec()
    }
}

fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(std::iter::once(0)).collect()
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
/*
//
//   FUNCTION: OnDestroy(HWND)
//
//   PURPOSE: Process the WM_DESTROY message to destroy fonts and free memory.
//   The application deletes the font and frees the application-defined
//   CHARMENUITEM structure for each menu item.
//
void OnDestroy(HWND hWnd)
{
    HMENU hMenu = GetMenu(hWnd);
    MENUITEMINFO mii = { sizeof(mii) };
    UINT uID = 0;
    PCHARMENUITEM pcmi = NULL;

    // Free resources associated with each menu item.
    for (uID = IDM_REGULAR; uID <= IDM_UNDERLINE; uID++)
    {
        // Get the item data.
        mii.fMask = MIIM_DATA;
        if (GetMenuItemInfo(hMenu, uID, FALSE, &mii))
        {
            pcmi = (PCHARMENUITEM)mii.dwItemData;

            // Destroy the font and free the item structure.
            DeleteObject(pcmi->hFont);
            LocalFree(pcmi);
        }
    }

    PostQuitMessage(0);
}
 */
static HUXTHEME: Lazy<HMODULE> =
  Lazy::new(|| unsafe { LoadLibraryA(s!("uxtheme.dll")).unwrap_or_default() });

  #[allow(dead_code)]
  fn allow_dark_mode_for_app(is_dark_mode: bool) {

    #[repr(C)]
    enum PreferredAppMode {
      Default,
      AllowDark,
      ForceDark,
      ForceLight,
      Max,
    }
    const UXTHEME_SETPREFERREDAPPMODE_ORDINAL: u16 = 135;
    type SetPreferredAppMode = unsafe extern "system" fn(PreferredAppMode) -> PreferredAppMode;
    static SET_PREFERRED_APP_MODE: Lazy<Option<SetPreferredAppMode>> = Lazy::new(|| unsafe {
      if HUXTHEME.is_invalid() {
        return None;
      }

      GetProcAddress(
        *HUXTHEME,
        PCSTR::from_raw(UXTHEME_SETPREFERREDAPPMODE_ORDINAL as usize as *mut _),
      )
      .map(|handle| std::mem::transmute(handle))
    });

    if let Some(_set_preferred_app_mode) = *SET_PREFERRED_APP_MODE {
        unsafe { _set_preferred_app_mode( if is_dark_mode { PreferredAppMode::ForceDark } else { PreferredAppMode::ForceLight } ) };
    }


  }
