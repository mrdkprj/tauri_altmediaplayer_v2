use crate::menu;
use crate::settings::Settings;

use once_cell::sync::Lazy;
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
use windows::Win32::Foundation::HMODULE;
#[allow(unused_imports)]
use windows::Win32::Foundation::{POINT,HWND, RECT, BOOL, TRUE};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::UI::Controls::SetWindowTheme;
use windows_core::{s, PCSTR};
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
use std::os::windows::ffi::OsStrExt;
#[allow(unused_imports)]
use std::sync::OnceLock;
#[allow(unused_imports)]
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute,DWMWA_USE_IMMERSIVE_DARK_MODE};
#[allow(unused_imports)]
use std::ffi::c_void;

static CELL: OnceLock<HWND> = OnceLock::new();

pub fn create_player_menu(window:&tauri::WebviewWindow, settings:&Settings) -> tauri::Result<()> {

    allow_dark_mode_for_app(true);
    // let window_ = window.clone();
    // menu::on_menu_event(Box::new( move |id| {
    //     window_.emit_to(tauri::EventTarget::WebviewWindow { label: id.label.clone() }, "contextmenu-event", id).unwrap();
    // }));

    // menu::create_player_menu(window, settings)
    //menu::create_player_menu(window, settings)?;
    change_test(window);
    Ok(())
}

pub fn create_playlist_menu(window:&tauri::WebviewWindow, _settings:&Settings) -> tauri::Result<()> {
    menu::create_playlist_menu(window)?;
    Ok(())
}

struct ICoreWebView2Ext (ICoreWebView2);

impl ICoreWebView2Ext {
    pub fn cast<T:ComInterface>(&self) -> Result<T, windows_core::Error>{
        self.0.cast::<T>()
    }
}

fn change_test(window:&tauri::WebviewWindow){

    // allow_dark_mode_for_app(true);
    // refresh_immersive_color_policy_state();

    CELL.set(window.hwnd().unwrap()).unwrap();
    window.with_webview( move |webview| {
        unsafe{

            let core_webview = ICoreWebView2Ext(webview.controller().CoreWebView2().unwrap());

            let core_webview_13: ICoreWebView2_11 = core_webview.cast::<ICoreWebView2_11>().unwrap();

            let handler = ContextMenuRequestedEventHandler::create(Box::new(handler2));
            let mut token: EventRegistrationToken = EventRegistrationToken::default();
            core_webview_13.add_ContextMenuRequested(&handler, &mut token).unwrap();

        }
    }).unwrap();
}

#[allow(non_snake_case)]
fn handler2(_sender: Option<ICoreWebView2>, args: Option<ICoreWebView2ContextMenuRequestedEventArgs>) -> Result<(), windows_core::Error>{
    println!("{}","enter");
    if args.is_none() {
        return Ok(());
    }

    unsafe {

        let args = args.unwrap();

        args.SetHandled(true)?;

        let menu:HMENU = CreatePopupMenu()?;

        //let item:HMENU = CreateMenu()?;
        let label:PCWSTR = PCWSTR::from_raw(pwstr_from_str("Test").as_ptr());
        println!("{}",string_from_pcwstr(&label));
        AppendMenuW(menu, MF_STRING, 100, label)?;
        let label2:PCWSTR = PCWSTR::from_raw(pwstr_from_str("Test\tAlt+F4").as_ptr());
        AppendMenuW(menu, MF_CHECKED | MF_STRING , 200, label2)?;
        AppendMenuW(menu, MF_SEPARATOR, 200, label2)?;
        AppendMenuW(menu, MF_STRING, 300, label)?;
        AppendMenuW(menu, MF_STRING, 400, label)?;

        let mut locationInScreenCoordinates:POINT = POINT::default();
        args.Location(&mut locationInScreenCoordinates)?;

        let hwnd:HWND = *CELL.get().unwrap();
        // let DARK_THEME_NAME: Vec<u16> = encode_wide("DarkMode_Explorer");
        // let name = PCWSTR::from_raw(DARK_THEME_NAME.as_ptr());

        // SetWindowTheme(
        //     hwnd,
        //     name,
        //     PCWSTR::null()
        // )?;
        // let pvattribute: *const c_void = &TRUE as *const BOOL as *const c_void;
        // let cbattribute: u32 = std::mem::size_of::<BOOL>() as u32;
        // DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, pvattribute, cbattribute)?;
        let selected_command_id = TrackPopupMenu(
            menu,
            TPM_TOPALIGN | TPM_LEFTALIGN | TPM_RETURNCMD,
            locationInScreenCoordinates.x + 300,
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

static HUXTHEME: Lazy<HMODULE> =
  Lazy::new(|| unsafe { LoadLibraryA(s!("uxtheme.dll")).unwrap_or_default() });

  fn allow_dark_mode_for_app(_is_dark_mode: bool) {

    #[repr(C)]
    enum PreferredAppMode {
      Default,
      AllowDark,
      ForceDark,
      // ForceLight,
      // Max,
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
        unsafe { _set_preferred_app_mode(PreferredAppMode::ForceDark) };
    }


  }

  fn refresh_immersive_color_policy_state() {
    const UXTHEME_REFRESHIMMERSIVECOLORPOLICYSTATE_ORDINAL: u16 = 104;
    type RefreshImmersiveColorPolicyState = unsafe extern "system" fn();
    static REFRESH_IMMERSIVE_COLOR_POLICY_STATE: Lazy<Option<RefreshImmersiveColorPolicyState>> =
      Lazy::new(|| unsafe {
        if HUXTHEME.is_invalid() {
          return None;
        }

        GetProcAddress(
          *HUXTHEME,
          PCSTR::from_raw(UXTHEME_REFRESHIMMERSIVECOLORPOLICYSTATE_ORDINAL as usize as *mut _),
        )
        .map(|handle| std::mem::transmute(handle))
      });

    if let Some(_refresh_immersive_color_policy_state) = *REFRESH_IMMERSIVE_COLOR_POLICY_STATE {
      unsafe { _refresh_immersive_color_policy_state() }
    }
  }


fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(std::iter::once(0)).collect()
  }