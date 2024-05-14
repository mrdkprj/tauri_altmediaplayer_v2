use std::{ffi::c_void, mem::{size_of, transmute}, sync::atomic::{AtomicUsize, Ordering}};
use once_cell::sync::Lazy;
use serde::Serialize;
use windows::Win32::{Foundation::{COLORREF, HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM}, Graphics::Gdi::{BeginPaint, ClientToScreen, CreateFontIndirectW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, ExcludeClipRect, FillRect, FrameRect, GetDC, GetMonitorInfoW, GetObjectW, GetWindowDC, InflateRect, InvalidateRect, LineTo, MonitorFromPoint, MonitorFromWindow, MoveToEx, OffsetRect, PtInRect, ReleaseDC, ScreenToClient, SelectObject, SetBkMode, SetTextColor, UpdateWindow, DT_CALCRECT, DT_LEFT, DT_RIGHT, DT_SINGLELINE, DT_VCENTER, HBRUSH, HDC, HFONT, HGDIOBJ, HPEN, LOGFONTW, MONITORINFO, MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTONULL, PAINTSTRUCT, PS_SOLID, TRANSPARENT}, System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryA}, UI::{Controls::{CloseThemeData, DrawThemeBackgroundEx, OpenThemeDataEx, HTHEME, MC_CHECKMARKNORMAL, MENU_POPUPCHECK, MENU_POPUPSUBMENU, MSM_NORMAL, OTD_NONCLIENT}, Input::KeyboardAndMouse::{GetActiveWindow, ReleaseCapture, SendInput, SetCapture, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT}, Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass}, WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, DispatchMessageW, GetAncestor, GetClientRect, GetCursorPos, GetMessageW, GetParent, GetSystemMetrics, GetWindowLongPtrW, GetWindowRect, IsWindowVisible, KillTimer, PostMessageW, RegisterClassExW, SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindow, SystemParametersInfoW, TranslateMessage, WindowFromPoint, CS_DROPSHADOW, CS_HREDRAW, CS_VREDRAW, GA_ROOTOWNER, GWL_USERDATA, HCURSOR, HICON, HWND_TOP, MSG, NONCLIENTMETRICSW, SM_CXHSCROLL, SM_CXMENUCHECK, SM_CYMENU, SPI_GETNONCLIENTMETRICS, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SW_HIDE, SW_SHOWNOACTIVATE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, TIMERPROC, WM_APP, WM_DESTROY, WM_ERASEBKGND, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCMOUSEMOVE, WM_PAINT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_THEMECHANGED, WNDCLASSEXW, WS_CLIPSIBLINGS, WS_EX_TOOLWINDOW, WS_POPUP}}};
use windows_core::{s, w, PCSTR, PCWSTR};
use crate::util::{decode_wide, encode_wide, HIWORD, LOWORD};

static COUNTER:AtomicUsize = AtomicUsize::new(400);

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Theme {
    Dark,
    Light,
}

const SUBMENU_TIMEOUT_MSEC:u32 = 400;
const LR_BUTTON_SIZE:i32 = 25;

const WM_MENUSELECTED:u32 = WM_APP + 0x0002;
const WM_CLOSEMENU:u32 = WM_APP + 0x0003;
const WM_INACTIVATE:u32 = WM_APP + 0x0004;

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MENU_TYPE(pub i32);
pub const XMT_STRING:MENU_TYPE = MENU_TYPE(0);
pub const XMT_CHECKBOX:MENU_TYPE = MENU_TYPE(1);
pub const XMT_RADIO:MENU_TYPE = MENU_TYPE(2);
pub const XMT_SUBMENU:MENU_TYPE = MENU_TYPE(3);
pub const XMT_SEPARATOR:MENU_TYPE = MENU_TYPE(4);

#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct MenuItemState(pub i32);
pub const MENU_NORMAL:MenuItemState = MenuItemState(1);
pub const MENU_CHECKED:MenuItemState = MenuItemState(2);
pub const MENU_DISABLED:MenuItemState = MenuItemState(4);

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
    accelerator:Option<Vec<u16>>,
    name:Option<Vec<u16>>,
    state:MenuItemState,
    menu_type:MENU_TYPE,
    index:i32,
    top:i32,
    bottom:i32,
    submenu:Option<HWND>,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    index:usize,
    hwnd:HWND,
    pub id:String,
    pub label:String,
    pub value:String,
    pub accelerator:String,
    pub name:String,
    pub state:MenuItemState,
    pub menu_type:MENU_TYPE,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectedMenuItem {
    pub id:String,
    pub label:String,
    pub value:String,
    pub name:String,
    pub state:MenuItemState,
}

impl SelectedMenuItem {
    fn from(item: &InnerMenuItem) -> Self {
        Self {
            id: decode_wide(&item.id),
            label: decode_wide(&item.label),
            value: decode_wide(&item.value),
            name: if item.name.is_none() { String::new() } else { decode_wide(item.name.as_ref().unwrap()) },
            state: item.state.clone(),
        }
    }
}

#[derive(Debug, Clone)]
struct MenuData {
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
        menu
    }

    pub fn new_with_theme(parent:HWND, theme:Theme) -> Self {
        let mut menu = MenuX::default();
        let is_dark = theme == Theme::Dark;
        menu.parent = parent;
        menu.theme.is_dark = is_dark;
        menu.hwnd = create_container_hwnd(parent, is_dark).unwrap();
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

    pub fn items(&self) -> Vec<MenuItem> {
        let userdata = unsafe { GetWindowLongPtrW(self.hwnd, GWL_USERDATA) };
        let data = unsafe { transmute::<isize, &MenuData>(userdata) };
        data.items.iter().map(|item| MenuItem::new(self.hwnd, item)).collect()
    }

    pub fn theme(&self) -> Theme {
        let userdata = unsafe { GetWindowLongPtrW(self.hwnd, GWL_USERDATA) };
        let data = unsafe { transmute::<isize, &MenuData>(userdata) };
        if data.theme.is_dark { Theme::Dark } else { Theme::Light }
    }

    pub fn set_theme(self, theme:Theme){
        let is_dark = theme == Theme::Dark;
        on_theme_change(self.hwnd, Some(is_dark));
    }

    fn get_state(disabled:Option<bool>, checked:Option<bool>) -> MenuItemState {
        let mut state = MENU_NORMAL.0;
        if disabled.is_some() && disabled.unwrap() {
            state |= MENU_DISABLED.0;
        }

        if checked.is_some() && checked.unwrap() {
            state |= MENU_CHECKED.0;
        }

        MenuItemState(state)
    }

    pub fn text(&mut self, id:&str, label:&str, disabled:Option<bool>) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, "", None, None, Self::get_state(disabled, None), XMT_STRING));
        self
    }

    pub fn text_with_accelerator(&mut self, id:&str, label:&str, disabled:Option<bool>, accelerator:&str) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, "", Some(accelerator), None, Self::get_state(disabled, None), XMT_STRING));
        self
    }

    pub fn check(&mut self, id:&str, label:&str, value:&str, checked:bool, disabled:Option<bool>) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, value, None, None, Self::get_state(disabled, Some(checked)), XMT_CHECKBOX));
        self
    }

    pub fn check_with_accelerator(&mut self, id:&str, label:&str, value:&str, checked:bool, disabled:Option<bool>, accelerator:&str) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, value, Some(accelerator), None, Self::get_state(disabled, Some(checked)), XMT_CHECKBOX));
        self
    }

    pub fn radio(&mut self, id:&str, label:&str, value:&str, name:&str, checked:bool, disabled:Option<bool>) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, value, None, Some(name), Self::get_state(disabled, Some(checked)), XMT_RADIO));
        self
    }

    pub fn radio_with_accelerator(&mut self, id:&str, label:&str, value:&str, name:&str, checked:bool, disabled:Option<bool>, accelerator:&str) -> &Self {
        self.items.push(InnerMenuItem::new(id, label, value, Some(accelerator), Some(name), Self::get_state(disabled, Some(checked)), XMT_RADIO));
        self
    }

    pub fn submenu(&mut self, label:&str) -> Self {
        let mut item = InnerMenuItem::new(label, label, "", None, None, MENU_NORMAL, XMT_SUBMENU);
        let mut submenu = MenuX::new(self.hwnd);
        submenu.is_main = false;
        submenu.theme = self.theme.clone();
        submenu.size = self.size.clone();
        item.submenu = Some(submenu.hwnd);
        self.items.push(item);
        submenu
    }

    pub fn separator(&mut self) -> &Self {
        self.items.push(InnerMenuItem::new("","","",None, None, MENU_NORMAL, XMT_SEPARATOR));
        self
    }

    pub fn build(&mut self) -> Result<(), windows_core::Error> {

        let is_dark = should_apps_use_dark_mode();
        let mut width = self.size.horizontal_margin;
        let mut height = self.size.vertical_margin;

        for i in 0..self.items.len() {

            let item = &mut self.items[i];
            item.index = i as i32;

            item.top = height;
            let (item_width, item_height) = measure_item(self.hwnd, &self.size, &item, is_dark)?;
            item.bottom = item.top + item_height;

            width = std::cmp::max(width, item_width);
            height += item_height;

        }

        width += self.size.horizontal_margin;
        height += self.size.vertical_margin;

        width += self.size.border_width * 2;
        height += self.size.border_width * 2;
        self.width = width;
        self.height = height;

        let data = MenuData {
            main:self.is_main,
            items:self.items.clone(),
            htheme: if self.is_main { Some(unsafe { OpenThemeDataEx(self.hwnd, w!("Menu"), OTD_NONCLIENT) }) } else { None },
            win_subclass_id: if self.is_main { Some(COUNTER.fetch_add(1, Ordering::Relaxed)) } else { None },
            selected_index:-1,
            height,
            width,
            visible_submenu_index:-1,
            size:self.size.clone(),
            theme:self.theme.clone(),
        };

        if self.is_main {
            Self::attach_owner_subclass(self, data.win_subclass_id.unwrap());
        }

        unsafe { SetWindowLongPtrW(self.hwnd, GWL_USERDATA, Box::into_raw(Box::new(data)) as _) };

        Ok(())
    }

    pub fn popup_at(&self, x:i32, y:i32) -> Option<&SelectedMenuItem> {

        let pt = get_display_point(self.hwnd, x, y, self.width, self.height);
        unsafe {
            let _ = SetWindowPos(self.hwnd, HWND_TOP, pt.x, pt.y, self.width, self.height, SWP_ASYNCWINDOWPOS | SWP_NOOWNERZORDER | SWP_NOACTIVATE);
            ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            SetCapture(self.hwnd) ;
        };

        let mut msg = MSG::default();
        let mut selected_item:Option<&SelectedMenuItem> = None;

        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {

            if self.parent != unsafe { GetActiveWindow() } {
                let _ = unsafe { PostMessageW(self.hwnd, WM_INACTIVATE, WPARAM(0), LPARAM(0)) };
            }

            match msg.message {

                WM_MENUSELECTED => {
                    selected_item = Some(unsafe { transmute::<isize, &SelectedMenuItem>(msg.lParam.0) });
                    break;
                }

                WM_CLOSEMENU => {
                    break;
                }

                _ => {
                    unsafe { TranslateMessage(&msg) };
                    unsafe { DispatchMessageW(&msg) };
                }
            }

        }

        let _ = unsafe { ReleaseCapture() };

        unsafe { ShowWindow(self.hwnd, SW_HIDE) };

        selected_item

    }

    fn attach_owner_subclass(&self, id:usize) {
        unsafe {
            let ancestor = GetAncestor(self.parent, GA_ROOTOWNER);
            SetWindowSubclass(
                if ancestor.0 == 0 { self.parent } else { ancestor },
                Some(menu_owner_subclass_proc),
                id,
                Box::into_raw(Box::new(self.hwnd)) as _
            );
        }
    }

}

impl MenuItem {

    fn new(hwnd:HWND, item:&InnerMenuItem) -> Self {
        Self {
            index:item.index as usize,
            hwnd,
            id:decode_wide(&item.id),
            label:decode_wide(&item.label),
            value: decode_wide(&item.value),
            accelerator: if item.accelerator.is_none() { String::new() } else { decode_wide(item.accelerator.as_ref().unwrap()) },
            name: if item.name.is_none() { String::new() } else { decode_wide(item.name.as_ref().unwrap()) },
            state:item.state.clone(),
            menu_type:item.menu_type.clone(),
        }
    }

    fn get_data(&self) -> &mut MenuData {
        let userdata = unsafe { GetWindowLongPtrW(self.hwnd, GWL_USERDATA) };
        unsafe { transmute::<isize, &mut MenuData>(userdata) }
    }

    fn set_data(&self, data:&mut MenuData){
        unsafe { SetWindowLongPtrW(self.hwnd, GWL_USERDATA, transmute::<&mut MenuData, isize>(data)) };
    }

    pub fn checked(&self) -> bool {
        let data = Self::get_data(self);
        (data.items[self.index].state.0 & MENU_CHECKED.0) != 0
    }

    pub fn set_checked(&self, checked:bool){
        let data = Self::get_data(self);
        if checked {
            data.items[self.index].state.0 |= MENU_CHECKED.0;
        } else {
            data.items[self.index].state.0 &= !MENU_CHECKED.0;
        }
        Self::set_data(self, data);
    }

    pub fn disabled(&self) -> bool {
        let data = Self::get_data(self);
        (data.items[self.index].state.0 & MENU_DISABLED.0) != 0
    }

    pub fn set_disabled(&self, disabled:bool){
        let data = Self::get_data(self);
        if disabled {
            data.items[self.index].state.0 |= MENU_DISABLED.0;
        } else {
            data.items[self.index].state.0 &= !MENU_DISABLED.0;
        }
        Self::set_data(self, data);
    }

    pub fn set_label(&self, label:&str){
        let data = Self::get_data(self);
        data.items[self.index].label = encode_wide(label);
        Self::set_data(self, data);
    }
}

impl InnerMenuItem {

    pub fn new(id:&str, label:&str, value:&str, accelerator:Option<&str>, name:Option<&str>, state:MenuItemState, menu_type:MENU_TYPE) -> Self {
        match menu_type {
            XMT_CHECKBOX | XMT_RADIO => {
                Self {
                    id: encode_wide(id),
                    label:encode_wide(label),
                    value: encode_wide(value),
                    accelerator: if accelerator.is_some() { Some(encode_wide(accelerator.unwrap())) } else { None },
                    name: if name.is_some() { Some(encode_wide(name.unwrap())) } else { None },
                    state,
                    menu_type,
                    index:-1,
                    top:0,
                    bottom:0,
                    submenu:None,
                }
            }

            XMT_SUBMENU => {
                Self {
                    id: encode_wide(id),
                    label:encode_wide(label),
                    value:Vec::new(),
                    accelerator:None,
                    name:None,
                    state,
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
                    accelerator:None,
                    name:None,
                    state,
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
                    value:Vec::new(),
                    accelerator: if accelerator.is_some() { Some(encode_wide(accelerator.unwrap())) } else { None },
                    name:None,
                    state,
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

unsafe extern "system" fn default_window_proc(
    window: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {

    match msg {

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
            if data.main {
                RemoveWindowSubclass(window, Some(menu_owner_subclass_proc), data.win_subclass_id.unwrap());
                CloseThemeData(data.htheme.unwrap()).unwrap();
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
            on_paint(window, data, theme).unwrap();
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
                let checked = (data.items[index as usize].state.0 & MENU_CHECKED.0) != 0;
                toggle_checked(&mut data.items[index as usize], !checked);
            }

            // toggle radio checkbox
            if data.items[index as usize].menu_type == XMT_RADIO {
                toggle_radio(data, index as usize);
            }


            SetWindowLongPtrW(hwnd, GWL_USERDATA, transmute::<&mut MenuData, isize>(data));
            init_menu_data(window);
            let menu_item = SelectedMenuItem::from(&data.items[index as usize]);
            PostMessageW(hwnd, WM_MENUSELECTED, WPARAM(0), LPARAM(Box::into_raw(Box::new(menu_item)) as _)).unwrap();

            LRESULT(0)
        }

        WM_LBUTTONDOWN | WM_RBUTTONDOWN => {
            // If mouse down outside of menu, exit
            if get_hwnd_from_point(window, lparam).is_none() {
                init_menu_data(window);
                PostMessageW(window, WM_CLOSEMENU, WPARAM(0), LPARAM(0)).unwrap();
                // If mouse down at window concerned, send mouse input
                send_mouse_input(window, msg);
                return LRESULT(0);
            }
            DefWindowProcW(window, msg, wparam, lparam)
        }

        _ => {
            DefWindowProcW(window, msg, wparam, lparam)
        }
    }

}

fn send_mouse_input(hwnd:HWND, msg: u32){

    let mut count = 0;
    let mut parent = unsafe { GetParent(hwnd) };
    let mut pt = POINT::default();
    let _ = unsafe { GetCursorPos(&mut pt) };
    while parent.0 != 0 {
        let mut rect = RECT::default();
        let _ = unsafe { GetWindowRect(parent, &mut rect) };
        if unsafe { PtInRect(&mut rect as *const _ as _, pt) }.as_bool() {
            count += 1;
        }
        parent = unsafe { GetParent(parent) };
    }

    if count > 0 {
        println!("send");
        let mut flags = MOUSEEVENTF_VIRTUALDESK | MOUSEEVENTF_ABSOLUTE;
        flags |= if msg == WM_LBUTTONDOWN { MOUSEEVENTF_LEFTDOWN } else { MOUSEEVENTF_RIGHTDOWN };

        let input = INPUT{
            r#type:INPUT_MOUSE,
            Anonymous:INPUT_0 {
                mi: MOUSEINPUT{
                    dx:pt.x,
                    dy:pt.y,
                    mouseData:0,
                    dwFlags: flags,
                    time:0,
                    dwExtraInfo:0
                    }
            }
        };
        unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    }

}

unsafe extern "system" fn menu_owner_subclass_proc(
    window: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uidsubclass: usize,
    _dwrefdata: usize,
) -> LRESULT {
    match msg {

        WM_THEMECHANGED => {
            let hwnd = transmute::<usize, &HWND>(_dwrefdata);
            on_theme_change(*hwnd, None);
            DefSubclassProc(window, msg, wparam, lparam)
        }

        _ => {
            DefSubclassProc(window, msg, wparam, lparam)
        }
    }
}

fn measure_item(hwnd:HWND, size:&MenuSize, item_data:&InnerMenuItem, is_dark:bool) -> Result<(i32,i32), windows_core::Error> {

    let mut width = 0;
    let height;

    match item_data.menu_type {

        XMT_SEPARATOR => {
            // separator - use half system height and zero width
            height = unsafe { (GetSystemMetrics(SM_CYMENU) as i32 + 4) / 2 };
        },

        _ => {

            let dc:HDC = unsafe { GetDC(hwnd) };
            let menu_font = get_font(is_dark)?;
            let font:HFONT = unsafe { CreateFontIndirectW(&menu_font) };
            let old_font:HGDIOBJ = unsafe { SelectObject(dc, font) };
            let mut text_rect = RECT::default();

            let mut raw_text = item_data.label.clone();
            if item_data.accelerator.is_some() {
                raw_text.extend(item_data.accelerator.as_ref().unwrap());
            }

            unsafe { DrawTextW(dc, raw_text.as_mut_slice(), &mut text_rect, DT_SINGLELINE | DT_LEFT | DT_VCENTER | DT_CALCRECT) };
            unsafe { SelectObject(dc, old_font) };

            let mut cx = text_rect.right - text_rect.left;

            let mut log_font = LOGFONTW::default();
            unsafe { GetObjectW(font, size_of::<LOGFONTW>() as i32, Some(&mut log_font as *mut _ as *mut c_void)) };

            let mut cy = log_font.lfHeight;
            if cy < 0 {
                cy = -cy;
            }
            cy += size.item_vertical_padding;

            height = cy;

            cx += size.item_horizontal_padding * 2;
            cx += LR_BUTTON_SIZE * 2;
            // extra padding
            if item_data.accelerator.is_some() {
                cx += 30;
            }

            // Windows adds 1 to returned value
            cx -= unsafe { GetSystemMetrics(SM_CXMENUCHECK) - 1 };

            width = cx;

            unsafe { ReleaseDC(hwnd, dc) };
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

fn on_paint(hwnd:HWND, data:&MenuData, theme:HTHEME) -> Result<(), windows_core::Error> {

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

fn paint(dc:HDC, data:&MenuData, items:&Vec<InnerMenuItem>, theme:HTHEME) -> Result<(), windows_core::Error> {

    let scheme = get_color_scheme(data);
    let selected_color = unsafe { CreateSolidBrush(COLORREF(scheme.border)) };
    let normal_color = unsafe { CreateSolidBrush(COLORREF(scheme.background_color)) };

    for item in items {

        let mut item_rect = get_item_rect(data, item);

        let disabled = (item.state.0 & MENU_DISABLED.0) != 0;
        let checked = (item.state.0 & MENU_CHECKED.0) != 0;

        if item.index == data.selected_index {
            unsafe { FillRect(dc, &mut item_rect, selected_color) };
        }else{
            unsafe { FillRect(dc, &mut item_rect, normal_color) };
        }

        match item.menu_type {

            XMT_SEPARATOR => {
                draw_separator(dc, scheme, item_rect)?;
            }

            _ => {

                if checked {
                    let mut rect = RECT{ left: item_rect.left, top:item_rect.top, right:item_rect.left + LR_BUTTON_SIZE, bottom:item_rect.top + LR_BUTTON_SIZE };
                    // center vertically
                    unsafe { OffsetRect(&mut rect, 0, ((item_rect.bottom - item_rect.top) - (rect.bottom - rect.top)) / 2) };
                    let mut check_rect = rect.clone();
                    unsafe { InflateRect(&mut check_rect as *mut _ as *mut RECT, -1, -1) };
                    unsafe { DrawThemeBackgroundEx(theme, dc, MENU_POPUPCHECK.0, MC_CHECKMARKNORMAL.0, &mut check_rect, None)? };
                }

                let mut text_rect = item_rect.clone();
                text_rect.left += LR_BUTTON_SIZE;
                text_rect.right -= LR_BUTTON_SIZE;

                if item.menu_type == XMT_SUBMENU {
                    let mut arrow_rect  = item_rect.clone();
                    let arrow_size = unsafe { GetSystemMetrics(SM_CXHSCROLL) };
                    text_rect.right -= arrow_size;
                    arrow_rect.left = item_rect.right - arrow_size;

                    // center vertically
                    unsafe { OffsetRect(&mut arrow_rect as *mut _ as *mut RECT, 0, ((item_rect.bottom - item_rect.top) - (arrow_rect.bottom - arrow_rect.top)) / 2) };
                    unsafe { DrawThemeBackgroundEx(theme, dc, MENU_POPUPSUBMENU.0, MSM_NORMAL.0, &mut arrow_rect, None)? };

                }

                draw_menu_text(dc, scheme, &text_rect, item, data.theme.is_dark, disabled)?;
                unsafe { ExcludeClipRect(dc, item_rect.left, item_rect.top, item_rect.right, item_rect.bottom) };
            }
        }
    }

    unsafe { DeleteObject(selected_color) };
    unsafe { DeleteObject(normal_color); }

    Ok(())
}

fn draw_separator(dc:HDC, scheme:&ColorScheme, rect:RECT) -> Result<(), windows_core::Error> {

    let mut separator_rect = rect.clone();

    separator_rect.top += (rect.bottom - rect.top) / 2;

    let pen:HPEN = unsafe { CreatePen(PS_SOLID, 1, COLORREF(scheme.border)) };
    let old_pen:HGDIOBJ = unsafe { SelectObject(dc,pen) };
    unsafe { MoveToEx(dc, separator_rect.left, separator_rect.top, None) };
    unsafe { LineTo(dc, separator_rect.right, separator_rect.top) };
    unsafe { SelectObject(dc,old_pen) };

    Ok(())
}

fn draw_menu_text(dc:HDC, scheme:&ColorScheme, rect:&RECT, item:&InnerMenuItem, is_dark:bool, disabled:bool) -> Result<(), windows_core::Error> {

    let mut text_rect = rect.clone();

    unsafe { SetBkMode(dc, TRANSPARENT) };
    if disabled {
        unsafe { SetTextColor(dc, COLORREF(scheme.disabled)) };
    } else {
        unsafe { SetTextColor(dc, COLORREF(scheme.color)) };
    }

    let menu_font = get_font(is_dark)?;
    let font:HFONT = unsafe { CreateFontIndirectW(&menu_font) };
    let old_font:HGDIOBJ = unsafe { SelectObject(dc,font) };

    unsafe { DrawTextW(dc, &mut item.label.clone(), &mut text_rect, DT_SINGLELINE | DT_LEFT | DT_VCENTER) };

    if item.accelerator.is_some() {
        unsafe { SetTextColor(dc, COLORREF(scheme.disabled)) };
        unsafe { DrawTextW(dc, &mut item.accelerator.as_ref().unwrap().clone(), &mut text_rect, DT_SINGLELINE | DT_RIGHT | DT_VCENTER) };
    }

    unsafe { SelectObject(dc,old_font) };

    Ok(())
}

fn show_submenu(main:HWND, _newindex:i32){
    let proc: TIMERPROC = Some(delay_show_submenu);
    unsafe { SetTimer(main, 400 as usize, SUBMENU_TIMEOUT_MSEC, proc) };
}

unsafe extern "system" fn delay_show_submenu(main: HWND, _msg: u32, id: usize, _time: u32){

    KillTimer(main, id).unwrap();

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
    unsafe { ClientToScreen(hwnd, &mut pt) };
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
    let userdata = unsafe { GetWindowLongPtrW(hwnd, GWL_USERDATA) };
    let data = unsafe { transmute::<isize, &MenuData>(userdata) };
    let submenu = if data.visible_submenu_index >= 0 { data.items[data.visible_submenu_index as usize].submenu.unwrap() } else { HWND(0) };

    let pt = to_screen_point(hwnd, lparam);

    let window = unsafe { WindowFromPoint(pt) };

    if submenu.0 != 0 && window == submenu {
        return Some(submenu);
    }

    if hwnd == window {
        return Some(hwnd);
    }

    None
}

fn init_menu_data(window:HWND){
    let userdata = unsafe { GetWindowLongPtrW(window, GWL_USERDATA) };
    let data = unsafe { transmute::<isize, &mut MenuData>(userdata) };
    data.selected_index = -1;

    if data.visible_submenu_index >= 0 {
        let hwnd = data.items[data.visible_submenu_index as usize].submenu.unwrap();
        hide_submenu(hwnd);
    }

    data.visible_submenu_index = -1;
    unsafe { SetWindowLongPtrW(window, GWL_USERDATA, transmute::<&mut MenuData, isize>(data)) };
}

fn get_theme(hwnd:HWND, data:&MenuData) -> HTHEME {
    if data.htheme.is_some() {
        return data.htheme.unwrap();
    }

    let parent = unsafe { GetParent(hwnd) };
    let userdata = unsafe { GetWindowLongPtrW(parent, GWL_USERDATA) };
    let parent_data = unsafe { transmute::<isize, &MenuData>(userdata) };

    parent_data.htheme.unwrap()
}

fn toggle_checked(item:&mut InnerMenuItem, checked:bool){
    if checked {
        item.state.0 |= MENU_CHECKED.0
    }else{
        item.state.0 &= !MENU_CHECKED.0;
    }
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

fn on_theme_change(hwnd:HWND, force_dark:Option<bool>){

    let is_dark = if force_dark.is_some() { force_dark.unwrap() } else { should_apps_use_dark_mode() };
    allow_dark_mode_for_window(hwnd, is_dark);
    let userdata = unsafe { GetWindowLongPtrW(hwnd, GWL_USERDATA) };
    let data = unsafe { transmute::<isize, &mut MenuData>(userdata) };
    let old_theme = data.htheme.unwrap();
    unsafe { CloseThemeData(old_theme).unwrap() };
    let theme = unsafe { OpenThemeDataEx(hwnd, w!("Menu"), OTD_NONCLIENT) };
    data.htheme = Some(theme);

    data.theme.is_dark = is_dark;
    unsafe { SetWindowLongPtrW(hwnd, GWL_USERDATA, transmute::<&mut MenuData, isize>(data)) };
    unsafe { UpdateWindow(hwnd) };

    for item in &data.items {
        if item.menu_type == XMT_SUBMENU {
            let submenu = item.submenu.unwrap();
            let userdata = unsafe { GetWindowLongPtrW(submenu, GWL_USERDATA) };
            let data = unsafe { transmute::<isize, &mut MenuData>(userdata) };
            data.theme.is_dark = is_dark;
            unsafe { SetWindowLongPtrW(submenu, GWL_USERDATA, transmute::<&mut MenuData, isize>(data)) };
            unsafe { UpdateWindow(submenu) };
        }
    }

}

fn create_container_hwnd(parent: HWND, is_dark:bool) -> Result<HWND, windows_core::Error> {

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
    let ex_style = WS_EX_TOOLWINDOW;

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