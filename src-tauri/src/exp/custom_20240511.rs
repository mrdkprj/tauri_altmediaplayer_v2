#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
//#![allow(unused_variables)]
use std::{ffi::c_void, mem::{size_of, transmute}, ops::BitOrAssign, sync::atomic::{AtomicUsize, Ordering}};

use once_cell::sync::Lazy;
use serde::Serialize;
use windows::Win32::{Foundation::*, Graphics::Gdi::*, System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryA}, UI::{Input::KeyboardAndMouse::{GetActiveWindow, GetCapture, ReleaseCapture, SetCapture}, Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass}, WindowsAndMessaging::*}};
use windows_core::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use crate::{menu::Menu, util::{decode_wide, encode_wide, HIWORD, LOWORD, WM_APPTHEMECHANGE}};

static COUNTER:AtomicUsize = AtomicUsize::new(300);

const SUBMENU_TIMEOUT_MSEC:u32 = 400;
const LR_BUTTON_SIZE:i32 = 25;

const WM_INACTIVATE:u32 = WM_APP + 0x0004;
const WM_MENUSELECTED:u32 = WM_APP + 0x0002;
const WM_CLOSEMENU:u32 = WM_APP + 0x0003;

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct X_MENU_TYPE(pub i32);
pub const XMT_STRING:X_MENU_TYPE = X_MENU_TYPE(0);
pub const XMT_CHECKBOX:X_MENU_TYPE = X_MENU_TYPE(1);
pub const XMT_RADIO:X_MENU_TYPE = X_MENU_TYPE(2);
pub const XMT_SUBMENU:X_MENU_TYPE = X_MENU_TYPE(3);
pub const XMT_SEPARATOR:X_MENU_TYPE = X_MENU_TYPE(4);

#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct MenuItemState(pub i32);
pub const MENU_NORMAL:MenuItemState = MenuItemState(0);
pub const MENU_CHECKED:MenuItemState = MenuItemState(1);
pub const MENU_DISABLED:MenuItemState = MenuItemState(2);

struct DisplayPoint{
    x:i32,
    y:i32,
    rtl:bool,
    reverse:bool,
}

pub struct Config {
    size:MenuSize,
    theme:MenuTheme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            size:MenuSize::default(),
            theme:MenuTheme::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MenuSize {
    border_width:i32,
    vertical_margin:i32,
    horizontal_margin:i32,
    item_vertical_padding:i32,
    item_horizontal_padding:i32,
}

impl Default for MenuSize {
    fn default() -> Self {
        Self {
            border_width:1,
            vertical_margin:2,
            horizontal_margin:0,
            item_vertical_padding:12,
            item_horizontal_padding:5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MenuTheme {
    is_dark:bool,
    color:ThemeColor
}

impl Default for MenuTheme {
    fn default() -> Self {
        Self {
            is_dark:false,
            color:ThemeColor::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ThemeColor {
    dark:ColorScheme,
    light:ColorScheme,
}

impl Default for ThemeColor {
    fn default() -> Self {
        Self {
            dark:DARK_COLOR_SCHEME,
            light:LIGHT_COLOR_SCHEME
        }
    }
}

#[derive(Debug, Clone)]
struct ColorScheme {
    color:u32,
    border:u32,
    disabled:u32,
    background_color:u32,
}

const DARK_COLOR_SCHEME:ColorScheme = ColorScheme {
    color:0x0e7e0e0,
    border:0x0454545,
    disabled:0x00565659,
    background_color:0x0252526
};

const LIGHT_COLOR_SCHEME:ColorScheme = ColorScheme {
    color:0x00e0e0e,
    border:0x0e9e2e2,
    disabled:0x00565659,
    background_color:0x00f5f5f5
};

#[derive(Debug, Clone)]
pub struct MenuX {
    parent:HWND,
    pub hwnd:HWND,
    items:Vec<InnerMenuItem>,
    width:i32,
    height:i32,
    is_main:bool,
    size:MenuSize,
    theme:MenuTheme
}

impl Default for MenuX {
    fn default() -> Self {
        Self {
            parent:HWND(0),
            hwnd: HWND(0),
            items:Vec::new(),
            height:0,
            width:0,
            is_main:true,
            size:MenuSize::default(),
            theme:MenuTheme::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct InnerMenuItem {
    id:Vec<u16>,
    label:Vec<u16>,
    value:Vec<u16>,
    name:Option<Vec<u16>>,
    state:MenuItemState,
    menu_type:X_MENU_TYPE,
    index:i32,
    top:i32,
    bottom:i32,
    submenu:Option<HWND>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MenuItem {
    pub id:String,
    pub label:String,
    pub value:String,
    pub state:MenuItemState,
}

impl MenuItem {
    fn from(item: &InnerMenuItem) -> Self {
        Self {
            id: decode_wide(&item.id),
            label: decode_wide(&item.label),
            value: decode_wide(&item.value),
            state: item.state.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct MenuData {
    parent:HWND,
    main:bool,
    items:Vec<InnerMenuItem>,
    htheme:Option<HTHEME>,
    win_subclass_id:Option<usize>,
    selected_index:i32,
    width:i32,
    height:i32,
    visible_submenu_index:i32,
    size:MenuSize,
    theme:MenuTheme,
}

impl MenuX {

    pub fn new(parent:HWND) -> Self {
        let mut menu = MenuX::default();
        menu.parent = parent;
        menu.hwnd = create_container_hwnd(parent, false).unwrap();
        println!("submenu.hwnd:{:?}",menu.hwnd);
        menu
    }

    pub fn new_with_theme(parent:HWND, is_dark:bool) -> Self {
        let mut menu = MenuX::default();
        menu.parent = parent;
        menu.theme.is_dark = is_dark;
        menu.hwnd = create_container_hwnd(parent, is_dark).unwrap();
        println!("menu.hwnd:{:?}",menu.hwnd);
        menu
    }

    pub fn new_from_config(parent:HWND, config:Config) -> Self {
        let mut menu = MenuX::default();
        let is_dark = config.theme.is_dark;
        menu.parent = parent;
        menu.theme = config.theme;
        menu.size = config.size;
        menu.hwnd = create_container_hwnd(parent, is_dark).unwrap();
        menu
    }

    pub fn is_dark(&self) -> bool {
        unsafe {
            let userdata = GetWindowLongPtrW(self.hwnd, GWL_USERDATA);
            let data = transmute::<isize, &MenuData>(userdata);
            data.theme.is_dark
        }
    }

    pub fn set_theme(self, is_dark:bool){
        on_theme_change(self.hwnd, Some(is_dark));
    }

    pub fn text(&mut self, id:&str, label:&str) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, "", None, None, XMT_STRING));
        self
    }

    pub fn check(&mut self, id:&str, label:&str, value:&str, checked:bool) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, value, None, Some(checked), XMT_CHECKBOX));
        self
    }

    pub fn radio(&mut self, id:&str, label:&str, value:&str, name:&str, checked:bool) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, value, Some(name), Some(checked), XMT_RADIO));
        self
    }

    pub fn submenu(&mut self, label:&str) -> Self {
        let mut item = InnerMenuItem::new(label, label, "", None, None, XMT_SUBMENU);
        let mut submenu = MenuX::new(self.hwnd);
        submenu.is_main = false;
        submenu.theme = self.theme.clone();
        submenu.size = self.size.clone();
        item.submenu = Some(submenu.hwnd);
        self.items.push(item);
        submenu
    }

    pub fn separator(&mut self) -> &Self {
        self.items.push(InnerMenuItem::new("", "", "", None, None, XMT_SEPARATOR));
        self
    }

    pub fn build(&mut self) -> Result<()> {

        let is_dark = should_apps_use_dark_mode();
        let mut width = 0;
        let mut height = 2;

        for i in 0..self.items.len() {

            let item = &mut self.items[i];
            item.index = i as i32;

            item.top = height;
            let (item_width, item_height) = measure_item(self.hwnd, &self.size, &item, is_dark)?;
            item.bottom = item.top + item_height;

            width = std::cmp::max(width, item_width);
            height += item_height;

        }

        height += 2;

        width += self.size.border_width * 2;
        height += self.size.border_width * 2;
        self.width = width;
        self.height = height;

        let data = MenuData {
            parent:self.parent,
            main:self.is_main,
            items:self.items.clone(),
            htheme: if self.is_main { Some(unsafe { OpenThemeDataEx(self.hwnd, w!("Menu"), OTD_NONCLIENT) }) } else { None },
            win_subclass_id: if self.is_main { Some(COUNTER.fetch_add(1, Ordering::Relaxed)) } else { None },
            selected_index:-1,
            height:height,
            width:width,
            visible_submenu_index:-1,
            size:self.size.clone(),
            theme:self.theme.clone(),
        };

        if self.is_main {
            Self::attach_menu_subclass_for_hwnd(self, data.win_subclass_id.unwrap());
        }

        unsafe { SetWindowLongPtrW(self.hwnd, GWL_USERDATA, Box::into_raw(Box::new(data)) as _) };

        Ok(())
    }

    pub fn popup_at(&self, x:i32, y:i32) -> Option<&MenuItem> {
        unsafe {

            let pt = get_display_point(self.hwnd, x, y, self.width, self.height);
            SetWindowPos(self.hwnd, HWND::default(), pt.x, pt.y, self.width, self.height, SWP_ASYNCWINDOWPOS | SWP_NOOWNERZORDER | SWP_NOACTIVATE).unwrap();

            ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            //ShowWindow(self.hwnd, SW_SHOWNORMAL);
            let parent = GetParent(self.parent) ;
            if parent.0 != 0 {
                println!("parent:{:?}",parent);
                SetCapture(parent);
            }else{
                println!("self.parent:{:?}",self.parent);
                SetCapture(self.parent);
            }
            //SetCapture(self.hwnd);
            SetCapture(self.parent);

            let mut msg = MSG::default();
            let mut selected_item:Option<&MenuItem> = None;

            while GetMessageW(&mut msg, None, 0, 0).as_bool() {

                if self.parent != GetActiveWindow() {
                    // Post message to initialize menu data
                    let _ = PostMessageW(self.hwnd, WM_INACTIVATE, WPARAM(0), LPARAM(0));
                }

                match msg.message {

                    WM_MENUSELECTED => {
                        selected_item = Some(transmute::<isize, &MenuItem>(msg.lParam.0));
                        break;
                    }

                    WM_CLOSEMENU => {
                        break;
                    }

                    // change hwnd to popup menu
                    WM_MOUSEMOVE | WM_NCMOUSEMOVE | WM_LBUTTONUP | WM_RBUTTONUP | WM_LBUTTONDOWN | WM_RBUTTONDOWN => {

                        msg.hwnd = self.hwnd;
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }

                    // no need to handle. so break.
                    WM_LBUTTONDBLCLK | WM_RBUTTONDBLCLK | WM_MBUTTONDOWN | WM_MBUTTONUP | WM_MBUTTONDBLCLK |
                    WM_NCLBUTTONDOWN | WM_NCLBUTTONUP | WM_NCLBUTTONDBLCLK | WM_NCRBUTTONDOWN | WM_NCRBUTTONUP |
                    WM_NCRBUTTONDBLCLK | WM_NCMBUTTONDOWN | WM_NCMBUTTONUP | WM_NCMBUTTONDBLCLK => {
                        println!("nontclient");
                        break;
                    }

                    _ => {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                     }
                }

            }

            let _ = ReleaseCapture();

            ShowWindow(self.hwnd, SW_HIDE);

            selected_item

        }
    }

    fn attach_menu_subclass_for_hwnd(&self, id:usize) {
        unsafe {
            let parent = GetParent(self.parent);
            let owner = if parent.0 == 0 { self.parent } else { parent };

            SetWindowSubclass(
                owner,
                Some(menu_subclass_proc),
                id,
                Box::into_raw(Box::new(self.hwnd)) as _
            );
        }
    }
}

impl InnerMenuItem {

    pub fn new(id:&str, label:&str, value:&str, name:Option<&str>, checked:Option<bool>, menu_type:X_MENU_TYPE) -> Self {
        match menu_type {
            XMT_CHECKBOX | XMT_RADIO => {
                Self {
                    id: encode_wide(id),
                    label:encode_wide(label),
                    value:encode_wide(value),
                    name: if name.is_some() { Some(encode_wide(name.unwrap())) } else { None },
                    state: if checked.unwrap() { MENU_CHECKED } else { MENU_NORMAL },
                    menu_type,
                    index:-1,
                    top:0,
                    bottom:0,
                    submenu:None,
                }
            }

            XMT_SUBMENU => {
                Self {
                    id: Vec::new(),
                    label:encode_wide(label),
                    value:Vec::new(),
                    name:None,
                    state:MENU_NORMAL,
                    menu_type,
                    index:-1,
                    top:0,
                    bottom:0,
                    submenu:None,
                }
            }

            XMT_SEPARATOR =>  {
                Self {
                    id: Vec::new(),
                    label:Vec::new(),
                    value:Vec::new(),
                    name:None,
                    state:MENU_NORMAL,
                    menu_type,
                    index:-1,
                    top:0,
                    bottom:0,
                    submenu:None,
                }
            }

            _ => {
                Self {
                    id: encode_wide(id),
                    label:encode_wide(label),
                    value:encode_wide(value),
                    name:None,
                    state:MENU_NORMAL,
                    menu_type,
                    index:-1,
                    top:0,
                    bottom:0,
                    submenu:None,
                }
            }
        }
    }
}

fn measure_item(hwnd:HWND, size:&MenuSize, item_data:&InnerMenuItem, is_dark:bool) -> Result<(i32,i32)> {

    let mut width = 0;
    let height;
    unsafe {

        match item_data.menu_type {

            XMT_SEPARATOR => {
                // separator - use half system height and zero width
                height = (GetSystemMetrics(SM_CYMENU) as i32 + 4) / 2;
            },

            _ => {

                let dc:HDC = GetDC(hwnd);
                let menu_font = get_font(is_dark)?;
                let font:HFONT = CreateFontIndirectW(&menu_font);
                let old_font:HGDIOBJ = SelectObject(dc, font);
                let mut text_rect = RECT::default();

                let mut raw_text = item_data.label.clone();

                DrawTextW(dc, raw_text.as_mut_slice(), &mut text_rect, DT_SINGLELINE | DT_LEFT | DT_VCENTER | DT_CALCRECT);
                SelectObject(dc, old_font);

                let mut cx = text_rect.right - text_rect.left;

                let mut log_font = LOGFONTW::default();
                GetObjectW(font, size_of::<LOGFONTW>() as i32, Some(&mut log_font as *mut _ as *mut c_void));

                let mut cy = log_font.lfHeight;
                if cy < 0 {
                    cy = -cy;
                }
                cy += size.item_vertical_padding;

                height = cy;

                cx += size.item_horizontal_padding * 2;
                cx += LR_BUTTON_SIZE * 2;
                // extra padding
                let text = PCWSTR::from_raw(item_data.label.as_ptr()).to_string().unwrap();
                if text.contains("\t") {
                    cx += 30;
                }

                // Windows adds 1 to returned value
                cx -= GetSystemMetrics(SM_CXMENUCHECK) - 1;

                width = cx;

                ReleaseDC(hwnd, dc);
            }
        }
    }

    Ok((width,height))
}

fn get_font(is_dark:bool) -> windows_core::Result<LOGFONTW> {

    let mut info:NONCLIENTMETRICSW = NONCLIENTMETRICSW::default();
    info.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
    unsafe { SystemParametersInfoW(SPI_GETNONCLIENTMETRICS, size_of::<NONCLIENTMETRICSW>() as u32, Some(&mut info as *mut _ as *mut c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0))? };

    let mut menu_font = info.lfMenuFont;
    if is_dark {
        // bold font
        menu_font.lfWeight = 700;
    }

    Ok(menu_font)
}

fn get_color_scheme(data:&MenuData) -> &ColorScheme {
    if data.theme.is_dark { &data.theme.color.dark } else { &data.theme.color.light }
}

fn paint_background(hwnd:HWND, data:&MenuData) {
    unsafe {

        let dc = GetWindowDC(hwnd);

        if dc.0 == 0 {
            return;
        }

        let mut client_rect = RECT::default();
        GetClientRect(hwnd, &mut client_rect).unwrap();

        let scheme = get_color_scheme(data);
        let hbr = CreateSolidBrush(COLORREF(scheme.background_color));
        FillRect(dc, &mut client_rect, hbr);
        DeleteObject(hbr);

        let hbr = CreateSolidBrush(COLORREF(scheme.border));
        FrameRect(dc, &mut client_rect, hbr);
        DeleteObject(hbr);

        ReleaseDC(hwnd, dc);
    }
}

fn ColorPick_OnPaint(hwnd:HWND, data:&MenuData, theme:HTHEME) -> Result<()> {

    let mut ps = PAINTSTRUCT::default();
    let dc = unsafe { BeginPaint(hwnd, &mut ps) };

    if dc.0 == 0 {
        return Ok(());
    }

    let index = index_from_rect(data, ps.rcPaint);

    if index.is_none() {
        paint(dc, data, &data.items, theme)?;
    } else {
        paint(dc, data, &vec![data.items[index.unwrap() as usize].clone()], theme)?;
    }

    unsafe { EndPaint(hwnd, &mut ps) };

    Ok(())
}

fn paint(dc:HDC, data:&MenuData, items:&Vec<InnerMenuItem>, theme:HTHEME) -> Result<()> {

    unsafe {

        let scheme = get_color_scheme(data);
        let selected_color = CreateSolidBrush(COLORREF(scheme.border));
        let normal_color = CreateSolidBrush(COLORREF(scheme.background_color));

        for item in items {

            let mut item_rect = get_item_rect(data, item);

            let _disabled = item.state == MENU_DISABLED;
            let checked = item.state == MENU_CHECKED;

            if item.index == data.selected_index {
                FillRect(dc, &mut item_rect, selected_color);
            }else{
                FillRect(dc, &mut item_rect, normal_color);
            }

            match item.menu_type {
                XMT_SEPARATOR => {
                    draw_separator(dc, scheme, item_rect)?;
                },
                _ => {

                    if checked{
                        let mut rect = RECT{ left: item_rect.left, top:item_rect.top, right:item_rect.left + LR_BUTTON_SIZE, bottom:item_rect.top + LR_BUTTON_SIZE };
                        // center vertically
                        OffsetRect(&mut rect, 0, ((item_rect.bottom - item_rect.top) - (rect.bottom - rect.top)) / 2);
                        let mut check_rect = rect.clone();
                        InflateRect(&mut check_rect as *mut _ as *mut RECT, -1, -1);
                        DrawThemeBackgroundEx(theme, dc, MENU_POPUPCHECK.0, MC_CHECKMARKNORMAL.0, &mut check_rect, None)?;
                    }

                    let mut text_rect = item_rect.clone();
                    text_rect.left += LR_BUTTON_SIZE;
                    text_rect.right -= LR_BUTTON_SIZE;

                    if item.menu_type == XMT_SUBMENU {
                        let mut arrow_rect  = item_rect.clone();
                        let arrow_size = GetSystemMetrics(SM_CXHSCROLL);
                        text_rect.right -= arrow_size;
                        arrow_rect.left = item_rect.right - arrow_size;

                        // center vertically
                        OffsetRect(&mut arrow_rect as *mut _ as *mut RECT, 0, ((item_rect.bottom - item_rect.top) - (arrow_rect.bottom - arrow_rect.top)) / 2);
                        DrawThemeBackgroundEx(theme, dc, MENU_POPUPSUBMENU.0, MSM_NORMAL.0, &mut arrow_rect, None)?;

                    }

                    draw_menu_text(dc, scheme, &text_rect, PCWSTR::from_raw(item.label.as_ptr()), data.theme.is_dark)?;
                    ExcludeClipRect(dc, item_rect.left, item_rect.top, item_rect.right, item_rect.bottom);
                }
            }
        }

        DeleteObject(selected_color);
        DeleteObject(normal_color);

    }

    Ok(())
}

fn draw_separator(dc:HDC, scheme:&ColorScheme, rect:RECT) -> Result<()> {
    unsafe {
        let mut separator_rect = rect.clone();

        separator_rect.top += (rect.bottom - rect.top) / 2;

        let pen:HPEN = CreatePen(PS_SOLID, 1, COLORREF(scheme.border));
        let old_pen:HGDIOBJ = SelectObject(dc,pen);
        MoveToEx(dc, separator_rect.left, separator_rect.top, None);
        LineTo(dc, separator_rect.right, separator_rect.top);
        SelectObject(dc,old_pen);
    }

    Ok(())
}

fn draw_menu_text(dc:HDC, scheme:&ColorScheme, rect:&RECT, raw_text:PCWSTR, is_dark:bool) -> Result<()> {
    unsafe {

        let text = raw_text.to_string().unwrap();
        let has_tab = text.contains("\t");
        let texts = text.split("\t").collect::<Vec::<&str>>();

        let mut text_rect = rect.clone();
        let mut first = encode_wide(texts[0]);
        let mut second = if has_tab { encode_wide(texts[1]) } else { encode_wide("") };

        SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, COLORREF(scheme.color));

        let menu_font = get_font(is_dark)?;
        let font:HFONT = CreateFontIndirectW(&menu_font);
        let old_font:HGDIOBJ = SelectObject(dc,font);

        DrawTextW(dc, &mut first, &mut text_rect, DT_SINGLELINE | DT_LEFT | DT_VCENTER);

        SetTextColor(dc, COLORREF(scheme.disabled));
        DrawTextW(dc, &mut second, &mut text_rect, DT_SINGLELINE | DT_RIGHT | DT_VCENTER);

        SelectObject(dc,old_font);

    }

    Ok(())
}

fn show_submenu(main:HWND, _newindex:i32){
    let proc: TIMERPROC = Some(delay_show_submenu);
    unsafe { SetTimer(main, 400 as usize, SUBMENU_TIMEOUT_MSEC, proc) };
}

unsafe extern "system" fn delay_show_submenu(main: HWND, _uMsg: u32, nIDEvent: usize, _dwTime: u32){
    KillTimer(main, nIDEvent).unwrap();

    let userdata = GetWindowLongPtrW(main, GWL_USERDATA);
    let data = transmute::<isize, &MenuData>(userdata);

    if data.visible_submenu_index >= 0 {
        let mut rect = RECT::default();
        GetWindowRect(main, &mut rect).unwrap();
        let item = &data.items[data.visible_submenu_index as usize];
        let hwnd = item.submenu.unwrap();
        let userdata = GetWindowLongPtrW(hwnd, GWL_USERDATA);
        let data = transmute::<isize, &MenuData>(userdata);

        let pt = get_display_point(hwnd, rect.right, rect.top + item.top, data.width, data.height);

        let x = if pt.rtl { rect.left - data.width } else { rect.right - 5 };
        let y = if pt.reverse {
            let mut newpt = POINT {x:0, y:item.bottom - data.height};
            ClientToScreen(main, &mut newpt);
            newpt.y
        } else {
            rect.top + item.top
        };

        SetWindowPos(hwnd, HWND_TOP, x, y, data.width, data.height, SWP_NOACTIVATE | SWP_NOOWNERZORDER).unwrap();
        ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }
}

fn hide_submenu(hwnd:HWND){
    unsafe {
        let userdata = GetWindowLongPtrW(hwnd, GWL_USERDATA);
        let data = transmute::<isize, &mut MenuData>(userdata);
        data.selected_index = -1;
        SetWindowLongPtrW(hwnd, GWL_USERDATA, transmute::<&mut MenuData, isize>(data));
        ShowWindow(hwnd, SW_HIDE)
    };
}

fn toggle_submenu(data:&mut MenuData, newindex:i32) -> bool {

    let mut should_show_submenu = false;

    if newindex < 0 {
        return should_show_submenu;
    }

    if data.visible_submenu_index >= 0 && data.visible_submenu_index != newindex {
        let hwnd = data.items[data.visible_submenu_index as usize].submenu.unwrap();
        hide_submenu(hwnd);
        data.visible_submenu_index = -1;
    }

    if data.visible_submenu_index < 0 && data.items[newindex as usize].menu_type == XMT_SUBMENU {
        data.visible_submenu_index = newindex;
        should_show_submenu = true;
    }

    should_show_submenu

}

fn get_display_point(hwnd:HWND, x:i32, y:i32, cx:i32, cy:i32) -> DisplayPoint {

    let mut rtl = false;
    let mut reverse = false;

    /*
    *  First get the dimensions of the monitor that contains (x, y).
    */
    let mut ppt = POINT::default();
    ppt.x = x;
    ppt.y = y;

    let mut hmon = unsafe { MonitorFromPoint(ppt, MONITOR_DEFAULTTONULL) };

    /*
    *  If (x, y) is not on any monitor, then use the monitor that
    *  the owner window is on.
    */
    if hmon.0 == 0 {
        hmon = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    }

    let mut minf = MONITORINFO::default();
    minf.cbSize = size_of::<MONITORINFO>() as u32;
    unsafe { GetMonitorInfoW(hmon, &mut minf) };

    /*
    *  If too high, then slide down.
    */
    if ppt.y < minf.rcWork.top {
        ppt.y = minf.rcMonitor.top;
    }

    /*
    *  If too far left, then slide right.
    */
    if ppt.x < minf.rcWork.left {
        ppt.x = minf.rcMonitor.left;
    }

    /*
    *  If too low, then slide up.
    */
    if ppt.y + cy >= minf.rcWork.bottom {
        ppt.y -= cy;
        reverse = true;
    }

    /*
    *  If too far right, then flip left.
    */
    //if ppt.x > minf.rcWork.right - cx {
    if ppt.x + cx >= minf.rcWork.right{
        ppt.x -= cx;
        rtl = true;
    }

    DisplayPoint {
        x:ppt.x,
        y:ppt.y,
        rtl,
        reverse,
    }
}

fn on_mouse_move(data:&mut MenuData, hwnd:HWND, screen_point:POINT) -> bool {

    let mut should_show_submenu = false;
    let selected_index = index_from_point(hwnd, screen_point, data);

    if data.visible_submenu_index >= 0 && selected_index < 0 {
        return should_show_submenu;
    }

    if data.selected_index != selected_index {

        should_show_submenu = toggle_submenu(data, selected_index);

        if selected_index >= 0 {
            let item = &data.items[selected_index as usize];
            let mut rc = get_item_rect(data, item);
            unsafe { InvalidateRect(hwnd, Some(&mut rc), false) };
        }

        if data.selected_index >= 0 {
            let item = &data.items[data.selected_index as usize];
            let mut rc = get_item_rect(data, item);
            unsafe { InvalidateRect(hwnd, Some(&mut rc), false) };
        }

    };

    data.selected_index = selected_index;

    should_show_submenu
}

fn get_item_rect(data:&MenuData, item:&InnerMenuItem) -> RECT {
    let border_width = data.size.border_width;
    RECT{ left:border_width, top:item.top + border_width, right: data.width - border_width, bottom: item.bottom + border_width}
}

fn to_screen_point(hwnd:HWND, lparam:LPARAM) -> POINT {
    let mut pt = POINT::default();
    pt.x = LOWORD(lparam.0 as u32) as i32;
    pt.y = HIWORD(lparam.0 as u32) as i32;
     let parent = unsafe { GetParent(hwnd)};
    unsafe { ClientToScreen(parent, &mut pt) };
    pt
}

fn index_from_rect(data:&MenuData, rect:RECT) -> Option<i32> {

    if rect.top == 0 && rect.bottom == data.height {
        return None;
    }

    for item in &data.items {
        if rect.top == item.top && rect.bottom == item.bottom {
            return Some(item.index);
        }
    }

    None
}

fn index_from_point(hwnd:HWND, screen_pt:POINT, data:&MenuData) -> i32 {
    let mut selected_index = -1;
    let mut pt = screen_pt.clone();
    unsafe { ScreenToClient(hwnd, &mut pt)};

    if pt.x >= 0 && pt.x < data.width && pt.y >= 0 && pt.y < data.height {
        for item in &data.items {
            if pt.y >= item.top && pt.y <= item.bottom {

                if item.menu_type != XMT_SEPARATOR {
                    selected_index = item.index;
                }

            }
        }
    }
    selected_index
}

fn get_hwnd_from_point(hwnd:HWND, lparam:LPARAM) -> Option<HWND> {
    unsafe {
        let userdata = GetWindowLongPtrW(hwnd, GWL_USERDATA);
        let data = transmute::<isize, &MenuData>(userdata);
        let submenu = if data.visible_submenu_index >= 0 { data.items[data.visible_submenu_index as usize].submenu.unwrap() } else { HWND(0) };

        let pt = to_screen_point(hwnd, lparam);

        let window = WindowFromPoint(pt);

        if submenu.0 != 0 && window == submenu {
            return Some(submenu);
        }

        if hwnd == window {
            return Some(hwnd);
        }

        None
    }
}

fn init_menu_data(window:HWND){
    unsafe {
        let userdata = GetWindowLongPtrW(window, GWL_USERDATA);
        let data = transmute::<isize, &mut MenuData>(userdata);
        data.selected_index = -1;

        if data.visible_submenu_index >= 0 {
            let hwnd = data.items[data.visible_submenu_index as usize].submenu.unwrap();
            hide_submenu(hwnd);
        }

        data.visible_submenu_index = -1;
        SetWindowLongPtrW(window, GWL_USERDATA, transmute::<&mut MenuData, isize>(data));
    }
}

fn get_theme(hwnd:HWND, data:&MenuData) -> HTHEME {
    if data.htheme.is_some() {
        return data.htheme.unwrap();
    }

    unsafe {
        let parent = GetParent(hwnd);
        let userdata = GetWindowLongPtrW(parent, GWL_USERDATA);
        let parent_data = transmute::<isize, &MenuData>(userdata);
        parent_data.htheme.unwrap()
    }
}

fn toggle_checked(item:&mut InnerMenuItem, checked:bool){
    item.state = if checked { MENU_CHECKED } else { MENU_NORMAL };
}

fn toggle_radio(data:&mut MenuData, index:usize){

    toggle_checked(&mut data.items[index], true);

    for i in 0..data.items.len() {
        if data.items[i].menu_type == XMT_RADIO && data.items[i].name.as_ref().unwrap() == data.items[index].name.as_ref().unwrap() {
            if i != index {
                toggle_checked(&mut data.items[i], false);
            }
        }
    }
}

fn send_mouse_input(hwnd:HWND, msg:u32, lparam:LPARAM){
    // let mut pt = POINT::default();
    // pt.x = LOWORD(lparam.0 as u32) as i32;
    // pt.y = HIWORD(lparam.0 as u32) as i32;
    // println!("pt:{:?}", pt);
    // unsafe { ScreenToClient(hwnd, &mut pt) };
    // println!("pt2:{:?}", pt);
    // let parent = unsafe { GetParent(hwnd) };
    // pt = to_screen_point(parent, lparam);
    // println!("pt3:{:?}", pt);
    // let window = unsafe { WindowFromPoint(pt)};
    // println!("window:{:?}", window);
    // let active = unsafe { GetActiveWindow() };
    // println!("hwnd:{:?}", hwnd);
    // println!("parent:{:?}", parent);
    // println!("active:{:?}", active);
    // if window.0 == 0 {
    //     return;
    // }
    // if window == parent || unsafe { IsChild(window, parent).as_bool() } {
    //     println!("enter");
    //     let pt = to_screen_point(window, lparam);
    //     let input = INPUT{
    //         r#type:INPUT_MOUSE,
    //         Anonymous:INPUT_0 {
    //             mi: MOUSEINPUT{
    //                 dx:pt.x,
    //                 dy:pt.y,
    //                 mouseData:0,
    //                 dwFlags:MOUSEEVENTF_LEFTDOWN,
    //                 time:0,
    //                 dwExtraInfo:0
    //                 }
    //         }
    //     };
    //     unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    // }
    let pt = to_screen_point(hwnd, lparam);
    let input = INPUT{
        r#type:INPUT_MOUSE,
        Anonymous:INPUT_0 {
            mi: MOUSEINPUT{
                dx:pt.x,
                dy:pt.y,
                mouseData:0,
                dwFlags: if msg == WM_LBUTTONUP { MOUSEEVENTF_LEFTDOWN } else { MOUSEEVENTF_RIGHTDOWN },
                time:0,
                dwExtraInfo:0
                }
        }
    };
    unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
}

unsafe extern "system" fn default_window_proc(
    window: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {

    match msg {

        // WM_ACTIVATE => {
        //     println!("act");
        //     if LOWORD(wparam.0 as u32) as u32 == WA_INACTIVE {
        //         if IsWindowVisible(window).as_bool() {
        //             init_menu_data(window);
        //             PostMessageW(window, WM_CLOSEMENU, WPARAM(0), LPARAM(0)).unwrap();
        //         }
        //         return LRESULT(0);
        //     }
        //     DefWindowProcW(window, msg, wparam, lparam)
        // }

        WM_INACTIVATE => {
            if IsWindowVisible(window).as_bool() {
                init_menu_data(window);
                PostMessageW(window, WM_CLOSEMENU, WPARAM(0), LPARAM(0)).unwrap();
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            let userdata = GetWindowLongPtrW(window, GWL_USERDATA);
            let data = transmute::<isize, &mut MenuData>(userdata);
            if data.htheme.is_some() {
                RemoveWindowSubclass(window, Some(menu_subclass_proc), data.win_subclass_id.unwrap());
                unsafe { CloseThemeData(data.htheme.unwrap()).unwrap() };
            }
            DefWindowProcW(window, msg, wparam, lparam)
        }

        WM_ERASEBKGND => {
            let userdata = GetWindowLongPtrW(window, GWL_USERDATA);
            let data = transmute::<isize, &MenuData>(userdata);
            paint_background(window, data);
            LRESULT(1)
        }

        WM_PAINT => {
            let userdata = GetWindowLongPtrW(window, GWL_USERDATA);
            let data = transmute::<isize, &mut MenuData>(userdata);
            let theme = get_theme(window, data);
            ColorPick_OnPaint(window, data, theme).unwrap();
            LRESULT(0)
        }

        WM_MOUSEMOVE | WM_NCMOUSEMOVE => {
            let userdata = GetWindowLongPtrW(window, GWL_USERDATA);
            let data = transmute::<isize, &mut MenuData>(userdata);
            let should_show_submenu = on_mouse_move(data, window, to_screen_point(window, lparam));
            SetWindowLongPtrW(window, GWL_USERDATA, transmute::<&mut MenuData, isize>(data));

            if should_show_submenu {
                show_submenu(window, 0);
            }

            if data.visible_submenu_index >= 0 {
                let hwnd = data.items[data.visible_submenu_index as usize].submenu.unwrap();
                let userdata = GetWindowLongPtrW(hwnd, GWL_USERDATA);
                let data = transmute::<isize, &mut MenuData>(userdata);
                on_mouse_move(data, hwnd, to_screen_point(window, lparam));
                SetWindowLongPtrW(hwnd, GWL_USERDATA, transmute::<&mut MenuData, isize>(data));
            }

            LRESULT(0)
        }

        WM_LBUTTONUP | WM_RBUTTONUP => {

            let hwnd_opt = get_hwnd_from_point(window, lparam);
            if hwnd_opt.is_none() {
                return LRESULT(0);
            }

            let hwnd = hwnd_opt.unwrap();
            let userdata = GetWindowLongPtrW(hwnd, GWL_USERDATA);
            let data = transmute::<isize, &mut MenuData>(userdata);
            let index = index_from_point(hwnd, to_screen_point(window, lparam), data);

            // toggle checkbox
            if data.items[index as usize].menu_type == XMT_CHECKBOX {
                let checked = data.items[index as usize].state == MENU_CHECKED;
                toggle_checked(&mut data.items[index as usize], !checked);
            }

            // toggle radio checkbox
            if data.items[index as usize].menu_type == XMT_RADIO {
                toggle_radio(data, index as usize);
            }


            SetWindowLongPtrW(hwnd, GWL_USERDATA, transmute::<&mut MenuData, isize>(data));
            init_menu_data(window);
            let menu_item = MenuItem::from(&data.items[index as usize]);
            PostMessageW(hwnd, WM_MENUSELECTED, WPARAM(0), LPARAM(Box::into_raw(Box::new(menu_item)) as _)).unwrap();

            LRESULT(0)
        }

        WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
            if get_hwnd_from_point(window, lparam).is_none() {
                init_menu_data(window);
                PostMessageW(window, WM_CLOSEMENU, WPARAM(0), LPARAM(0)).unwrap();
                return LRESULT(0);
            }
            DefWindowProcW(window, msg, wparam, lparam)
        }

        _ => {
            DefWindowProcW(window, msg, wparam, lparam)
        }
    }

}

fn on_theme_change(hwnd:HWND, force_dark:Option<bool>){
    unsafe {

        let is_dark = if force_dark.is_some() { force_dark.unwrap() } else { should_apps_use_dark_mode() };
        allow_dark_mode_for_window(hwnd, is_dark);
        let userdata = GetWindowLongPtrW(hwnd, GWL_USERDATA);
        let data = transmute::<isize, &mut MenuData>(userdata);
        let old_theme = data.htheme.unwrap();
        CloseThemeData(old_theme).unwrap();
        let theme = OpenThemeDataEx(hwnd, w!("Menu"), OTD_NONCLIENT);
        data.htheme = Some(theme);

        data.theme.is_dark = is_dark;
        SetWindowLongPtrW(hwnd, GWL_USERDATA, transmute::<&mut MenuData, isize>(data));
        UpdateWindow(hwnd);

        for item in &data.items {
            if item.menu_type == XMT_SUBMENU {
                let submenu = item.submenu.unwrap();
                let userdata = GetWindowLongPtrW(submenu, GWL_USERDATA);
                let data = transmute::<isize, &mut MenuData>(userdata);
                data.theme.is_dark = is_dark;
                SetWindowLongPtrW(submenu, GWL_USERDATA, transmute::<&mut MenuData, isize>(data));
                UpdateWindow(submenu);
            }
        }
    }
}

fn create_container_hwnd(parent: HWND, is_dark:bool) -> Result<HWND> {

    let class_name = w!("CUSTOM_POPUPMENU");

    let class = WNDCLASSEXW {
      cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
      style: CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW,
      lpfnWndProc: Some(default_window_proc),
      cbClsExtra: 0,
      cbWndExtra: 0,
      hInstance: unsafe { HINSTANCE(GetModuleHandleW(PCWSTR::null()).unwrap_or_default().0) },
      hIcon: HICON::default(),
      hCursor: HCURSOR::default(),
      hbrBackground: HBRUSH::default(),
      lpszMenuName: PCWSTR::null(),
      lpszClassName: class_name,
      hIconSm: HICON::default(),
    };

    unsafe { RegisterClassExW(&class) };

    let window_styles = WS_POPUP | WS_CLIPSIBLINGS;
    let ex_style = WS_EX_TOOLWINDOW | WS_EX_TOPMOST;

    let hwnd = unsafe {
      CreateWindowExW(
        ex_style,
        PCWSTR::from_raw(class_name.as_ptr()),
        PCWSTR::null(),
        window_styles,
        0,
        0,
        0,
        0,
        parent,
        None,
        GetModuleHandleW(PCWSTR::null()).unwrap_or_default(),
        None,
      )
    };

    allow_dark_mode_for_window(hwnd, is_dark);

    Ok(hwnd)
}

/*
* Subclass to handle theme change message
 */
unsafe extern "system" fn menu_subclass_proc(
    window: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uidsubclass: usize,
    _dwrefdata: usize,
) -> LRESULT {
    match msg {

        WM_THEMECHANGED => {
            unsafe {
                let hwnd = transmute::<usize, &HWND>(_dwrefdata);
                on_theme_change(*hwnd, None);
            }
            DefSubclassProc(window, msg, wparam, lparam)
        }

        _ => {
            DefSubclassProc(window, msg, wparam, lparam)
        }
    }
}

static HUXTHEME: Lazy<HMODULE> = Lazy::new(|| unsafe { LoadLibraryA(s!("uxtheme.dll")).unwrap_or_default() });

fn allow_dark_mode_for_window(hwnd: HWND, is_dark: bool) {
    const UXTHEME_ALLOWDARKMODEFORWINDOW_ORDINAL: u16 = 133;
    type AllowDarkModeForWindow = unsafe extern "system" fn(HWND, bool) -> bool;
    static ALLOW_DARK_MODE_FOR_WINDOW: Lazy<Option<AllowDarkModeForWindow>> = Lazy::new(|| unsafe {
        if HUXTHEME.is_invalid() {
            return None;
        }

        GetProcAddress(
            *HUXTHEME,
            PCSTR::from_raw(UXTHEME_ALLOWDARKMODEFORWINDOW_ORDINAL as usize as *mut _),
        )
        .map(|handle| std::mem::transmute(handle))
    });

    if let Some(_allow_dark_mode_for_window) = *ALLOW_DARK_MODE_FOR_WINDOW {
        unsafe { _allow_dark_mode_for_window(hwnd, is_dark) };
    }

}

fn should_apps_use_dark_mode() -> bool {
    const UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL: u16 = 132;
    type ShouldAppsUseDarkMode = unsafe extern "system" fn() -> bool;
    static SHOULD_APPS_USE_DARK_MODE: Lazy<Option<ShouldAppsUseDarkMode>> = Lazy::new(|| unsafe {
        if HUXTHEME.is_invalid() {
            return None;
        }

        GetProcAddress(
            *HUXTHEME,
            PCSTR::from_raw(UXTHEME_SHOULDAPPSUSEDARKMODE_ORDINAL as usize as *mut _),
        )
        .map(|handle| std::mem::transmute(handle))
    });

    SHOULD_APPS_USE_DARK_MODE.map(|should_apps_use_dark_mode| unsafe { (should_apps_use_dark_mode)() })
    .unwrap_or(false)
}