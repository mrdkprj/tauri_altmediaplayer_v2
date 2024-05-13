use once_cell::sync::Lazy;
use tauri::Manager;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2_13, COREWEBVIEW2_PREFERRED_COLOR_SCHEME_DARK,COREWEBVIEW2_PREFERRED_COLOR_SCHEME_LIGHT,
};
use windows::Win32::{Foundation::{HMODULE, LPARAM, WPARAM}, System::LibraryLoader::{GetProcAddress, LoadLibraryW}, UI::WindowsAndMessaging::{PostMessageW, WM_APP, WM_THEMECHANGED}};
use windows_core::{w, Interface, PCSTR, PCWSTR};
use std::{os::windows::ffi::OsStrExt, sync::atomic::{AtomicU32, Ordering}};
use crate::settings::Theme;

pub const WM_APPTHEMECHANGE:u32 = WM_APP + 0x0001;
static HUXTHEME: Lazy<HMODULE> = Lazy::new(|| unsafe { LoadLibraryW(w!("uxtheme.dll")).unwrap_or_default() });

pub fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}

pub fn decode_wide(wide: &Vec<u16>) -> String {
    let len = unsafe { windows::Win32::Globalization::lstrlenW(PCWSTR::from_raw(wide.as_ptr())) } as usize;
    let w_str_slice = unsafe { std::slice::from_raw_parts(wide.as_ptr(), len) };
    String::from_utf16_lossy(w_str_slice)
}

#[allow(non_snake_case)]
pub fn LOWORD(dword: u32) -> u16 {
  (dword & 0xFFFF) as u16
}

#[allow(non_snake_case)]
pub fn HIWORD(dword: u32) -> u16 {
  ((dword & 0xFFFF_0000) >> 16) as u16
}

pub fn init_apply_theme(window:&tauri::WebviewWindow, theme:Theme){
    change_webview_theme(window, theme);
    allow_dark_mode_for_app(theme);
}

pub fn change_theme(window:&tauri::WebviewWindow, theme:Theme){
    change_webview_theme(window, theme);
    allow_dark_mode_for_app(theme);
    unsafe { PostMessageW(window.hwnd().unwrap(), WM_APPTHEMECHANGE, WPARAM(theme as usize), LPARAM(0)).unwrap() };
    unsafe { PostMessageW(window.hwnd().unwrap(), WM_THEMECHANGED, WPARAM(0), LPARAM(0)).unwrap() };
}

fn change_webview_theme(window:&tauri::WebviewWindow, theme:Theme){
    let webview_theme = match theme {
        Theme::dark => COREWEBVIEW2_PREFERRED_COLOR_SCHEME_DARK,
        Theme::light => COREWEBVIEW2_PREFERRED_COLOR_SCHEME_LIGHT,
    };

    let windows = window.app_handle().webview_windows();

    windows.iter().enumerate().for_each(|(_,(_, win))|{
        win.with_webview( move |webview| {
            unsafe{
                let core_webview_13: ICoreWebView2_13 = webview.controller().CoreWebView2().unwrap().cast().unwrap();
                core_webview_13.Profile().unwrap().SetPreferredColorScheme(webview_theme).unwrap();
            }
        }).unwrap();
    })

}

fn allow_dark_mode_for_app(theme:Theme) {

    #[allow(dead_code)]
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
        unsafe { _set_preferred_app_mode( if theme == Theme::dark { PreferredAppMode::ForceDark } else { PreferredAppMode::ForceLight } ) };
    }

}

pub struct Counter(AtomicU32);

impl Counter {
    #[allow(unused)]
    pub const fn new() -> Self {
        Self(AtomicU32::new(1))
    }

    #[allow(unused)]
    pub const fn new_with_start(start: u32) -> Self {
        Self(AtomicU32::new(start))
    }

    pub fn next(&self) -> u32 {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
}
