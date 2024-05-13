#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
use std::{ffi::c_void, mem::{size_of, transmute}};

use windows::Win32::{Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW, UI::{Input::KeyboardAndMouse::{GetActiveWindow, GetCapture, ReleaseCapture, SetCapture}, WindowsAndMessaging::*}};
use windows_core::*;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use crate::util::{encode_wide, decode_wide, LOWORD, HIWORD};

#[allow(non_camel_case_types)]
#[derive(Clone, PartialEq, Eq, Debug)]
struct X_MENU_TYPE(pub i32);
const XMT_STRING:X_MENU_TYPE = X_MENU_TYPE(0);
const XMT_CHECKBOX:X_MENU_TYPE = X_MENU_TYPE(1);
const XMT_RADIO:X_MENU_TYPE = X_MENU_TYPE(2);
const XMT_SUBMENU:X_MENU_TYPE = X_MENU_TYPE(3);
const XMT_SEPARATOR:X_MENU_TYPE = X_MENU_TYPE(4);
const XMT_MARGIN:X_MENU_TYPE = X_MENU_TYPE(5);
const MENU_SUBCLASS_ID: usize = 200;

struct ColorSchema {
    color:u32,
    border:u32,
    disabled:u32,
}

const DARK_COLOR_SCHEMA:ColorSchema = ColorSchema {
    color:0xc7bfbf99,
    border:0x00565659,
    disabled:0x00565659,
};

const CXFAKEMENU:i32 = 500;
const ITEM_HEIGHT:i32 = 50;
const ITEM_COUNT:i32 = 3;
#[derive(Debug, Clone)]
struct MenuData {
    width:i32,
    height:i32,
    items:Vec<MenuItem>
}


pub struct MenuX {
    parent:HWND,
    hwnd:HWND,
    items:Vec<MenuItem>,
    menus:Vec<MenuX>
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    label:String,
    state:i32,
    menu_type:X_MENU_TYPE,
    top:i32,
    bottom:i32,
}

impl Default for MenuX {
    fn default() -> Self {
        Self {
            parent:HWND(0),
            hwnd: HWND(0),
            items:Vec::new(),
            menus:Vec::new(),
        }
    }
}

fn make_lparam(x: i16, y: i16) -> LPARAM {
    LPARAM(((x as u16 as u32) | ((y as u16 as u32) << 16)) as usize as _)
}

impl MenuX {
    pub fn new(parent:HWND) -> Self {

        Self {
            parent,
            hwnd:HWND(0),
            items:Vec::new(),
            menus:Vec::new(),
        }
    }

    pub fn append(&mut self, text:&str) -> &Self {
        self.items.push(MenuItem::new(text));
        self
    }

    pub fn build(&mut self){
        let mut width = 0;
        let mut height = 0;
        let mut items = Vec::new();

        for item in self.items.iter() {
            let (item_width, item_height) = measure_item(self.hwnd, item).unwrap();
            width = std::cmp::max(width, item_width);
            height += item_height;
            let mut item = item.clone();
            item.top = height;
            item.bottom = height + item_height;
            items.push(item.clone());
        }
        let data = MenuData {
            height,
            width,
            items,
        };
        self.hwnd = create_container_hwnd(self.parent, data).unwrap();

    }

    pub fn show(&self, x:i32, y:i32) {
        unsafe {
            let window_styles = WS_POPUP | WS_CLIPCHILDREN;
            let ex_style = WS_EX_TOOLWINDOW | WS_EX_WINDOWEDGE | WS_EX_TOPMOST;

            let mut rc = RECT{left:0,top:0, right:CXFAKEMENU, bottom:(ITEM_HEIGHT * self.items.len() as i32) as i32};
            AdjustWindowRectEx(&mut rc, window_styles, false, ex_style).unwrap();

            let cx = rc.right - rc.left;
            let cy = rc.bottom - rc.top;
            let mut pt = POINT::default();
            //ColorPick_ChooseLocation(parent, x, y, cx, cy, &pt);
            SetWindowPos(self.hwnd, HWND_TOP, x, y, cx, cy, SWP_NOACTIVATE).unwrap();

            ShowWindow(self.hwnd, SW_SHOW);
            SetCapture(self.hwnd);

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {

                /*
                *  Something may have happened that caused us to stop.
                *  If so, then stop.
                */
                //if (cps.fDone) break;

                /*
                *  If our owner stopped being the active window
                *  (e.g., the user Alt+Tab'd to another window
                *  in the meantime), then stop.
                */
                let hwndActive = GetActiveWindow();
                // println!("hwndactive:{:?}",hwndActive);
                // println!("self.parent:{:?}",self.parent);
                // println!("self.hwnd:{:?}",self.hwnd);
                if hwndActive != self.hwnd || GetCapture() != self.hwnd{
                    break;
                }

                match msg.message {

                    WM_MOUSEMOVE | WM_NCMOUSEMOVE => {
                        pt.x = LOWORD(msg.lParam.0 as u32) as i32;
                        pt.y = HIWORD(msg.lParam.0 as u32) as i32;
                        MapWindowPoints(msg.hwnd, self.hwnd, &mut [pt]);
                        msg.lParam = make_lparam(pt.x as i16, pt.y as i16);
                        msg.hwnd = self.hwnd;
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }

                    WM_LBUTTONDOWN |
                    WM_LBUTTONUP | WM_LBUTTONDBLCLK |
                    WM_RBUTTONDOWN |
                    WM_RBUTTONUP |
                    WM_RBUTTONDBLCLK |
                    WM_MBUTTONDOWN |
                    WM_MBUTTONUP |
                    WM_MBUTTONDBLCLK => {
                        pt.x = LOWORD(msg.lParam.0 as u32) as i32;
                        pt.y = HIWORD(msg.lParam.0 as u32) as i32;
                        MapWindowPoints(msg.hwnd, self.hwnd, &mut [pt]);
                        msg.lParam = make_lparam(pt.x as i16, pt.y as i16);
                        msg.hwnd = self.hwnd;
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                        break;
                    }

                    WM_NCLBUTTONDOWN |
                    WM_NCLBUTTONUP |
                    WM_NCLBUTTONDBLCLK |
                    WM_NCRBUTTONDOWN |
                    WM_NCRBUTTONUP |
                    WM_NCRBUTTONDBLCLK |
                    WM_NCMBUTTONDOWN |
                    WM_NCMBUTTONUP |
                    WM_NCMBUTTONDBLCLK => {
                        break;
                    }

                    _ => {

                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);

                        if hwndActive != self.hwnd || GetCapture() != self.hwnd{
                            break;
                        }

                     }
                }

            }

            /*
            *  Clean up the capture we created.
            */
            let _ = ReleaseCapture();

            ShowWindow(self.hwnd, SW_HIDE);
        };
    }
}

impl MenuItem {
    pub fn new(text:&str) -> Self {
        Self {
            label:text.to_string(),
            state:0,
            menu_type: XMT_STRING,
            top:0,
            bottom:0
        }
    }
}

impl MenuData {
    pub fn find(&self, y:i32) -> &MenuData {
        for item in self.items.iter(){
            if y >= item.top && y <= item.bottom {
                return item
            }
        }
    }
}

pub fn tryit(parent:HWND) -> MenuX {
    let mut menu = MenuX::new(parent);
    menu.append("text1");
    menu.append("text2");
    menu.append("text3");
    menu.build();
    menu
}

const V_MARGIN:i32 = 1;
const H_MARGIN:i32 = 2;
const BASE_SIZE:SIZE = SIZE {cx:16 + 2 * 3, cy:15 + 2 * 3};

fn measure_item(hwnd:HWND, item_data:&MenuItem) -> Result<(i32,i32)> {

    let mut width = 0;
    let mut height = 0;
    unsafe {

        match item_data.menu_type {

            XMT_SEPARATOR => {
                // separator - use half system height and zero width
                height = (GetSystemMetrics(SM_CYMENU) as i32 + 4) / 2;
            },

            XMT_MARGIN => {
                height = 2;
            },

            _ => {

                let dc:HDC = GetDC(hwnd);
                let menu_font = get_font()?;
                let font:HFONT = CreateFontIndirectW(&menu_font);
                let old_font:HGDIOBJ = SelectObject(dc, font);
                let mut text_rect = RECT::default();

                let mut raw_text = encode_wide(item_data.label);

                DrawTextW(dc, raw_text.as_mut_slice(), &mut text_rect, DT_SINGLELINE | DT_LEFT | DT_VCENTER | DT_CALCRECT);

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
                height = std::cmp::max(cy + 4, BASE_SIZE.cy);

                // L/R margin for readability
                cx += 2 * H_MARGIN;
                // space between button and menu text
                cx += V_MARGIN;
                // button width (L=button; R=empty margin)
                cx += 2 * BASE_SIZE.cx;
                // extra padding
                if item_data.label.contains("\t") {
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

fn get_font() -> windows_core::Result<LOGFONTW> {
    let mut info:NONCLIENTMETRICSW = NONCLIENTMETRICSW::default();
    info.cbSize = size_of::<NONCLIENTMETRICSW>() as u32;
    unsafe { SystemParametersInfoW(SPI_GETNONCLIENTMETRICS, size_of::<NONCLIENTMETRICSW>() as u32, Some(&mut info as *mut _ as *mut c_void), SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0))? };

    let mut menu_font = info.lfMenuFont;
    menu_font.lfWeight = 700;

    Ok(menu_font)
}

fn ColorPick_OnPaint(hwnd:HWND, data:&MenuData){
    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);

        if hdc.0 == 0 {
            return;
        }

        let mut cl = RECT::default();
        GetClientRect(hwnd, &mut cl).unwrap();
        println!("cl:{:?}",cl);
        for item in data.items.iter() {

            let top = item.top;
            let mut rc = RECT{left:0, top, right: data.width, bottom: item.bottom};
            let hbr = if item.state == 1 { CreateSolidBrush(COLORREF(DARK_COLOR_SCHEMA.color)) } else { CreateSolidBrush(COLORREF(DARK_COLOR_SCHEMA.border)) };
            if item.state == 1 {
                FillRect(hdc, &mut rc, hbr);
            }else{
                FillRect(hdc, &mut rc, hbr);
            }
            DeleteObject(hbr);
        }

        EndPaint(hwnd, &mut ps);
    }
}

fn ColorPick_OnMouseMove(data:&MenuData, hwnd:HWND, x:i32, y:i32) -> i32 {
    let mut selected_index = None;

    if x >= 0 && x < data.width && y >= 0 && y < data.height {
        println!("y:{:?}",y);
        selected_index = Some(data.find(y));
    }

    return ColorPick_ChangeSel(pcps, hwnd, selected_index);

}

fn ColorPick_ChangeSel(pcps:&MenuData, hwnd:HWND, selected_index:MenuItem) -> i32{
    /*
    *  If the selection changed, then
    *  repaint the items that need repainting.
    */
    unsafe {

        if pcps.selected_index != selected_index {

            if pcps.selected_index >= 0 || selected_index < 0{
                let mut rc = ColorPick_GetColorRect(pcps.selected_index);
                InvalidateRect(hwnd, Some(&mut rc), true);
            }


            if selected_index >= 0 {
                let mut rc = ColorPick_GetColorRect(selected_index);
                InvalidateRect(hwnd, Some(&mut rc), true);
            }
        }
    }

    selected_index
}

fn ColorPick_GetColorRect(item:MenuItem) -> RECT
{
    /*
    *  Build the "menu" item rect.
    */
    RECT{left:0, top: item.top, right:50, bottom:item.bottom}

}

unsafe extern "system" fn default_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {

    //let data = GetWindowLongPtrW(hwnd,GWL_USERDATA);

        //let item_data_ptr = data as *mut MenuData;
        //let pcps = &mut *item_data_ptr);
        //(*item_data_ptr).selected_index = 0;

    match msg {
        WM_CREATE => {
             let userdata = GetWindowLongPtrW(hwnd,GWL_USERDATA);
            if userdata == 0 {
                let createstruct = &*(lparam.0 as *const CREATESTRUCTW);
                let userdata = createstruct.lpCreateParams;
                let window_flags = Box::from_raw(userdata as *mut MenuData);
                SetWindowLongPtrW(hwnd, GWL_USERDATA, Box::into_raw(window_flags) as _);
              }
            LRESULT(0)
        }

        WM_PAINT => {
            let data = GetWindowLongPtrW(hwnd, GWL_USERDATA);
            let pcps = transmute::<isize, &mut MenuData>(data);
            ColorPick_OnPaint(hwnd, pcps);
            LRESULT(0)
        }

        WM_MOUSEMOVE | WM_NCMOUSEMOVE => {
            let data = GetWindowLongPtrW(hwnd, GWL_USERDATA);
            let pcps = transmute::<isize, &mut MenuData>(data);
            println!("WM_MOUSEMOVE:{:?}",pcps.selected_index);
            let abc = ColorPick_OnMouseMove(pcps, hwnd, LOWORD(lparam.0 as u32) as i32,HIWORD(lparam.0 as u32) as i32);
            pcps.selected_index = abc;
            SetWindowLongPtrW(hwnd, GWL_USERDATA, transmute::<&mut MenuData, isize>(pcps));
            LRESULT(0)
        }

        // WM_LBUTTONUP => {
        //     ColorPick_OnLButtonUp(pcps, hwnd, LOWORD(lparam.0 as u32) as i32,HIWORD(lparam.0 as u32) as i32);
        //     DefWindowProcW(hwnd, msg, wparam, lparam)
        // }

        // WM_SYSKEYDOWN | WM_KEYDOWN => {
        //     ColorPick_OnKeyDown(pcps, hwnd, wParam);
        // }

        _ => {

            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
    }

}

fn create_container_hwnd(parent: HWND, data:MenuData) -> Result<HWND> {

    let class_name = w!("WRY_WEBVIEWabc");

    let class = WNDCLASSEXW {
      cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
      style: CS_HREDRAW | CS_VREDRAW,
      lpfnWndProc: Some(default_window_proc),
      cbClsExtra: 0,
      cbWndExtra: 0,
      hInstance: unsafe { HINSTANCE(GetModuleHandleW(PCWSTR::null()).unwrap_or_default().0) },
      hIcon: HICON::default(),
      hCursor: HCURSOR::default(),
      hbrBackground: unsafe { CreateSolidBrush(COLORREF(DARK_COLOR_SCHEMA.color)) },
      lpszMenuName: PCWSTR::null(),
      lpszClassName: class_name,
      hIconSm: HICON::default(),
    };

    unsafe { RegisterClassExW(&class) };

    let window_styles = WS_POPUP | WS_POPUPWINDOW | WS_CLIPSIBLINGS;
    let ex_style = WS_EX_TOOLWINDOW | WS_EX_OVERLAPPEDWINDOW | WS_EX_TOPMOST;

    let hwnd = unsafe {
      CreateWindowExW(
        ex_style,
        PCWSTR::from_raw(class_name.as_ptr()),
        PCWSTR::null(),
        window_styles,
        0,
        0,
        data.width,
        data.height,
        parent,
        None,
        GetModuleHandleW(PCWSTR::null()).unwrap_or_default(),
        Some(Box::into_raw(Box::new(data)) as _),
      )
    };

    Ok(hwnd)
}
