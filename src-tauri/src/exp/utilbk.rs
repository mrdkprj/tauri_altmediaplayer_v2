
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
use windows::Win32::Foundation::{POINT,HWND, RECT, BOOL, TRUE};
#[allow(unused_imports)]
use windows_core::{ComInterface, PCWSTR, PWSTR, HRESULT};
#[allow(unused_imports)]
use windows::Win32::System::WinRT::EventRegistrationToken;
#[allow(unused_imports)]
use windows::Win32::UI::WindowsAndMessaging::{AppendMenuW, MENUITEMINFOW};
#[allow(unused_imports)]
use windows::Win32::UI::WindowsAndMessaging::TrackPopupMenu;
#[allow(unused_imports)]
use windows::Win32::UI::WindowsAndMessaging::{HMENU,CreatePopupMenu,GetMenuItemRect,SetMenuItemInfoW };
#[allow(unused_imports)]
use windows::Win32::UI::WindowsAndMessaging::{MENU_ITEM_FLAGS, MF_POPUP, MF_GRAYED, MF_CHECKED, MF_STRING, MF_BYPOSITION, MF_SEPARATOR, TPM_TOPALIGN, TPM_LEFTALIGN, TPM_RETURNCMD, MF_OWNERDRAW};
#[allow(unused_imports)]
use std::sync::OnceLock;
#[allow(unused_imports)]
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute,DWMWA_USE_IMMERSIVE_DARK_MODE};
#[allow(unused_imports)]
use std::ffi::c_void;

static CELL: OnceLock<HWND> = OnceLock::new();

#[derive(PartialEq)]
pub enum Theme {
    Dark = 2,
    Light = 1,
}
struct ICoreWebView2Ext (ICoreWebView2);

impl ICoreWebView2Ext {
    pub fn cast<T:ComInterface>(&self) -> Result<T, windows_core::Error>{
        self.0.cast::<T>()
    }
}

pub fn change_theme(window:&tauri::WebviewWindow, theme:Theme){

    window.with_webview( move |webview| {
        unsafe{
            let core_webview = ICoreWebView2Ext(webview.controller().CoreWebView2().unwrap());
            let core_webview_13: ICoreWebView2_13 = core_webview.cast::<ICoreWebView2_13>().unwrap();
            let value:COREWEBVIEW2_PREFERRED_COLOR_SCHEME = COREWEBVIEW2_PREFERRED_COLOR_SCHEME(theme as i32);
            core_webview_13.Profile().unwrap().SetPreferredColorScheme(value).unwrap();
        }
    }).unwrap();

}

pub fn change_menu(window:&tauri::WebviewWindow){

    //if CELL.get().is_none() {
        window.with_webview( move |webview| {
            unsafe{
                let mut parentwindow:HWND = HWND::default();
                webview.controller().ParentWindow(&mut parentwindow).unwrap();
                CELL.set(parentwindow).unwrap();

                let core_webview = ICoreWebView2Ext(webview.controller().CoreWebView2().unwrap());

                let core_webview_13: ICoreWebView2_11 = core_webview.cast::<ICoreWebView2_11>().unwrap();

                let handler = ContextMenuRequestedEventHandler::create(Box::new(handler2));
                let mut token: EventRegistrationToken = EventRegistrationToken::default();
                core_webview_13.add_ContextMenuRequested(&handler, &mut token).unwrap();

                // let core_webview_2 = core_webview.cast::<ICoreWebView2_2>().unwrap();
                // let env = core_webview_2.Environment().unwrap();
                // let env3 = env.cast::<ICoreWebView2Environment3>().unwrap();

                // CreateCoreWebView2CompositionControllerCompletedHandler::wait_for_async_operation(
                //     Box::new(move |handler| {
                //         env3
                //         .CreateCoreWebView2CompositionController(parentwindow, &handler)
                //         .map_err(webview2_com::Error::WindowsError)
                //     }),
                //     Box::new(move |error_code, _controller| {
                //       error_code?;
                //       Ok(())
                //     }),
                //   ).unwrap();

                //env3.CreateCoreWebView2CompositionController(parentwindow, &handler).unwrap();
            }
        }).unwrap();
    //}

}

fn handler2(sender: Option<ICoreWebView2>, args: Option<ICoreWebView2ContextMenuRequestedEventArgs>) -> Result<(), windows_core::Error>{
    if args.is_none() {
        return Ok(());
    }

    let args = args.unwrap();

    unsafe{
        //c.SendMouseInput(COREWEBVIEW2_MOUSE_EVENT_KIND_MOVE, COREWEBVIEW2_MOUSE_EVENT_VIRTUAL_KEYS_NONE, 0, POINT{x:0, y:0}).unwrap();
        //args.SetHandled(true)?;
        let core_webview_13 = sender.unwrap().cast::<ICoreWebView2_2>().unwrap();
        let env = core_webview_13.Environment().unwrap();
        let env9 = env.cast::<ICoreWebView2Environment9>().unwrap();

        let label:PCWSTR = PCWSTR::from_raw(pwstr_from_str("Test").as_ptr());
        let item = env9.CreateContextMenuItem(label, None, COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_COMMAND).unwrap();
        let selected_handler = CustomItemSelectedEventHandler::create(Box::new(handler3));
        let mut token = EventRegistrationToken::default();
        item.add_CustomItemSelected(&selected_handler, &mut token)?;
        let items = args.MenuItems()?;
        let mut count = 0;
        items.Count(&mut count)?;
        println!("{}", count);
        items.InsertValueAtIndex(count, &item)?;

        let c = count.clone();
        for _i in 0..c {
            let val = items.GetValueAtIndex(0)?;
            let mut name = PWSTR::null();
            val.Name(&mut name)?;
            println!("{}", name.to_string().unwrap());
            if name.to_string().unwrap() != "Test" {
                //println!("removed: {}", c);
                items.RemoveValueAtIndex(0)?;
            }

        }


    }

    Ok(())
}

fn handler3(_sender: Option<ICoreWebView2ContextMenuItem>, _args: Option<windows_core::IUnknown>) -> Result<(), windows_core::Error>{

    Ok(())
}
/*
#[allow(non_snake_case)]
fn handler2(_sender: Option<ICoreWebView2>, args: Option<ICoreWebView2ContextMenuRequestedEventArgs>) -> Result<(), windows_core::Error>{
    if args.is_none() {
        return Ok(());
    }
    println!("{}", "handler2");
    unsafe {

        let args = args.unwrap();

        args.SetHandled(true)?;

        let menu:HMENU = CreatePopupMenu()?;
        //let item:HMENU = CreateMenu()?;
        let label:PCWSTR = PCWSTR::from_raw(pwstr_from_str("Test").as_ptr());
        println!("{}",string_from_pcwstr(&label));
        AppendMenuW(menu, MF_STRING, 100, label)?;
        AppendMenuW(menu, MF_CHECKED | MF_STRING , 200, label)?;

        let mut locationInScreenCoordinates:POINT = POINT::default();
        args.Location(&mut locationInScreenCoordinates)?;

        let hwnd:HWND = *CELL.get().unwrap();
        // let pvattribute: *const c_void = &TRUE as *const BOOL as *const c_void;
        // let cbattribute: u32 = std::mem::size_of::<BOOL>() as u32;
        // DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, pvattribute, cbattribute)?;
        let selected_command_id = TrackPopupMenu(
            menu,
            TPM_TOPALIGN | TPM_LEFTALIGN | TPM_RETURNCMD,
            locationInScreenCoordinates.x + 100,
            locationInScreenCoordinates.y,
            0,
            hwnd,
            None);

        if selected_command_id.as_bool() {
            args.SetSelectedCommandId(selected_command_id.0)?;
        }

        let mut r:RECT = RECT::default();
        GetMenuItemRect(hwnd, menu, 1, &mut r)?;
        println!("{:?}",r);
    }
    Ok(())
}
*/
use windows::Win32::{
    Foundation::{WPARAM, LPARAM, LRESULT, POINT, RECT, HWND},
    Graphics::Gdi::{ScreenToClient, PtInRect},
    UI::Controls::WM_MOUSELEAVE,
    UI::Shell::{SetWindowSubclass, RemoveWindowSubclass, DefSubclassProc},
    UI::WindowsAndMessaging::{SetWindowsHookExA, WH_MOUSE_LL, UnhookWindowsHookEx, CallNextHookEx, GetClientRect, WM_MOUSEMOVE, MSLLHOOKSTRUCT, HHOOK},
  };
  use std::mem;
  use std::sync::Mutex;
  use once_cell::sync::Lazy;

  #[derive(Clone, serde::Serialize)]
  struct Position {
    x:i32,
    y:i32,
  }

  static FOWARDING_MOUSE_MESSAGES:Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
  static FORWARDING_WINDOWS: Lazy<Mutex<Option<&tauri::WebviewWindow>>> = Lazy::new(|| Mutex::new(None));
  static MOUSE_HOOK:Lazy<Mutex<Option<HHOOK>>> = Lazy::new(|| Mutex::new(None) );

  pub fn doit(){

  }

  pub fn clear_forward(){

      if let Some(window) = *FORWARDING_WINDOWS.lock().unwrap() {
          forward_mouse_messages(window, false);
      }
  }

  pub fn forward_mouse_messages(window:&tauri::WebviewWindow, forward:bool){

    let hwnd = window.hwnd().unwrap();
    let mut fowarding_mouse_messages = FOWARDING_MOUSE_MESSAGES.lock().unwrap();
    let mut forwarding_windows = FORWARDING_WINDOWS.lock().unwrap();
    let mut mouse_hook = MOUSE_HOOK.lock().unwrap();

    if forward {

      *fowarding_mouse_messages = true;
      *forwarding_windows = Some(window);

      unsafe{
        // Subclassing is used to fix some issues when forwarding mouse messages;
        // see comments in |SubclassProc|.
        SetWindowSubclass(
          hwnd,
          Some(sub_class_proc),
          1,
          Box::into_raw(Box::new(window)) as usize
        );

        if *mouse_hook == None {
          let rusult = SetWindowsHookExA(
            WH_MOUSE_LL,
            Some(mouse_hook_proc),
            None,
            0
          );
          *mouse_hook = rusult.ok();
        }
      }

    } else {

      if let Some(window) = *FORWARDING_WINDOWS.lock().unwrap() {
        *forwarding_windows = None;
        unsafe{
          RemoveWindowSubclass(hwnd, Some(sub_class_proc), 1);
        }
      }

      if *fowarding_mouse_messages && forwarding_windows.is_empty() {
        *fowarding_mouse_messages = false;
        unsafe{
          if *mouse_hook != None {
            UnhookWindowsHookEx(mouse_hook.unwrap());
          }
        }
        *mouse_hook = None;
      }

    }


  }

  unsafe extern "system" fn sub_class_proc(hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM, _uidsubclass: usize, _dwrefdata: usize)  -> LRESULT {

    if umsg == WM_MOUSELEAVE {
      // When input is forwarded to underlying windows, this message is posted.
      // If not handled, it interferes with Chromium logic, causing for example
      // mouseleave events to fire. If those events are used to exit forward
      // mode, excessive flickering on for example hover items in underlying
      // windows can occur due to rapidly entering and leaving forwarding mode.
      // By consuming and ignoring the message, we're essentially telling
      // Chromium that we have not left the window despite somebody else getting
      // the messages. As to why this is caught for the legacy window and not
      // the actual browser window is simply that the legacy window somehow
      // makes use of these events; posting to the main window didn't work.
      if *FOWARDING_MOUSE_MESSAGES.lock().unwrap() {
          return windows::Win32::Foundation::LRESULT(0);
      }

    }

    unsafe{
      return DefSubclassProc(hwnd, umsg, wparam, lparam);
    }
  }

  unsafe extern "system" fn mouse_hook_proc(n_code:i32,wparam: WPARAM, lparam: LPARAM) -> LRESULT{
    if n_code < 0 {
      return CallNextHookEx(None, n_code, wparam, lparam);
    }

    // Post a WM_MOUSEMOVE message for those windows whose client area contains
    // the cursor since they are in a state where they would otherwise ignore all
    // mouse input.

    if wparam.0 == WM_MOUSEMOVE as usize {

      let forwarding_windows = FORWARDING_WINDOWS.lock().unwrap();

      for window in &*forwarding_windows {
        let hwnd = window.hwnd().unwrap();

        // At first I considered enumerating windows to check whether the cursor
        // was directly above the window, but since nothing bad seems to happen
        // if we post the message even if some other window occludes it I have
        // just left it as is.
        let mut client_rect = RECT::default();
        GetClientRect(hwnd, &mut client_rect);

        let hook_struct = unsafe { mem::transmute::<LPARAM, &MSLLHOOKSTRUCT>(lparam) };
        let mut p:POINT = hook_struct.pt;
        //println!("{:?}", p);
        ScreenToClient(hwnd, &mut p as *mut POINT);
        if PtInRect(&mut client_rect, p) == true {
          //println!("Message from Rust: {}","here");
          //let w:WPARAM = WPARAM(0);  // No virtual keys pressed for our purposes
          //let l:LPARAM = make_lparam(p.x as i16, p.y as i16);
          //PostMessageW(hwnd, WM_MOUSEMOVE, w, l);
          window.emit("WM_MOUSEMOVE", Position{x:p.x, y:p.y}).unwrap();
        }
    }

  }

    return CallNextHookEx(None, n_code, wparam, lparam);
  }

  fn _make_lparam(x: i16, y: i16) -> LPARAM {
    LPARAM(((x as u16 as u32) | ((y as u16 as u32) << 16)) as usize as _)
  }

