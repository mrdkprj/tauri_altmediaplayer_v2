#![allow(dead_code)]
#![allow(unused_imports)]
use windows::Win32::{Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*};
use windows_core::*;
use crate::util::{encode_wide, decode_wide};

pub struct MenuX {
    hwnd:HWND,
    items:Vec<MenuItem>,
    menus:Vec<MenuX>
}

pub struct MenuItem {
    hwnd:HWND,
    id:String,
}

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

impl Default for MenuX {
    fn default() -> Self {
        Self {
            hwnd: HWND(0),
            items:Vec::new(),
            menus:Vec::new(),
        }
    }
}

impl MenuX {
    pub fn new(parent:HWND) -> Self {
        let hwnd = create_container_hwnd(parent).unwrap();
        Self {
            hwnd,
            items:Vec::new(),
            menus:Vec::new(),
        }
    }

    pub fn append(&mut self, text:&str) -> &Self {
        self.items.push(MenuItem::new(self.hwnd, text));
        self
    }

    pub fn show(&self) {
        unsafe {
            let dc = GetWindowDC(self.hwnd);
            let mut lprect = RECT::default();
            GetWindowRect(self.hwnd, &mut lprect).unwrap();
            println!("{:?}", lprect);
            FillRect(dc, &mut lprect, CreateSolidBrush(COLORREF(DARK_COLOR_SCHEMA.color)));

            self.items.iter().enumerate().for_each(|(index,item)| {
                let mut rect = RECT::default();
                rect.top = (index as i32) * 100;
                rect.bottom = rect.top + 100;
                rect.left = 0;
                rect.right = 500;
                let dc = GetWindowDC(item.hwnd);
                FillRect(dc, &mut lprect, CreateSolidBrush(COLORREF(DARK_COLOR_SCHEMA.border)));
            });
            ShowWindow(self.hwnd, SW_SHOW)
        };
    }
}

impl MenuItem {
    pub fn new(parent:HWND, text:&str) -> Self {
        let hwnd = create_item(parent);
        Self {
            hwnd,
            id:text.to_string()
        }
    }
}

pub fn tryit(parent:HWND) -> MenuX {
    let mut menu = MenuX::new(parent);
    menu.append("text1");
    menu.append("text2");
    menu.append("text3");
    menu
}

unsafe extern "system" fn default_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn create_container_hwnd(parent: HWND) -> Result<HWND> {

    let class_name = w!("WRY_WEBVIEW");

    let class = WNDCLASSEXW {
      cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
      style: CS_HREDRAW | CS_VREDRAW,
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

    let window_styles = WS_CHILD | WS_CLIPCHILDREN;

    let hwnd = unsafe {
      CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        PCWSTR::from_raw(class_name.as_ptr()),
        PCWSTR::null(),
        window_styles,
        0,
        0,
        500,
        500,
        parent,
        None,
        GetModuleHandleW(PCWSTR::null()).unwrap_or_default(),
        None,
      )
    };

    Ok(hwnd)
}

fn create_item(parent:HWND) -> HWND {
    unsafe {
    let hwnd =
        CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        PCWSTR::from_raw(encode_wide("BUTTON").as_ptr()),
        PCWSTR::null(),
        WS_CHILD | WINDOW_STYLE(BS_PUSHBUTTON as u32),  // Styles
        0,         // x position
        0,         // y position
        500,        // Button width
        100,        // Button height
        parent,     // Parent window
        None,       // No menu.
        GetModuleHandleW(PCWSTR::null()).unwrap_or_default(),
        None);      // Pointer not needed.
        hwnd
    }

}