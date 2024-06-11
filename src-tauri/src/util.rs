use crate::settings::Theme;
use once_cell::sync::Lazy;
use tauri::Manager;
use webview2_com::{
    AcceleratorKeyPressedEventHandler,
    Microsoft::Web::WebView2::Win32::{ICoreWebView2AcceleratorKeyPressedEventArgs, ICoreWebView2AcceleratorKeyPressedEventArgs2, ICoreWebView2Controller, ICoreWebView2_13, COREWEBVIEW2_PREFERRED_COLOR_SCHEME_DARK, COREWEBVIEW2_PREFERRED_COLOR_SCHEME_LIGHT},
};
use windows::{
    core::{w, Error, Interface, PCSTR},
    Win32::{
        Foundation::{BOOL, HMODULE, LPARAM, WPARAM},
        System::{
            LibraryLoader::{GetProcAddress, LoadLibraryW},
            WinRT::EventRegistrationToken,
        },
        UI::WindowsAndMessaging::{PostMessageW, WM_THEMECHANGED},
    },
};

static HUXTHEME: Lazy<HMODULE> = Lazy::new(|| unsafe { LoadLibraryW(w!("uxtheme.dll")).unwrap_or_default() });

pub fn init_webview(window: &tauri::WebviewWindow, theme: Theme) {
    for webview_window in window.webview_windows().values() {
        webview_window
            .with_webview(|webview| {
                let mut token = EventRegistrationToken::default();
                let handler = AcceleratorKeyPressedEventHandler::create(Box::new(on_accelerator_keypressed));
                unsafe { webview.controller().add_AcceleratorKeyPressed(&handler, &mut token).unwrap() };
            })
            .unwrap();
    }

    change_webview_theme(window, theme);
    allow_dark_mode_for_app(theme);
}

fn on_accelerator_keypressed(_: Option<ICoreWebView2Controller>, args: Option<ICoreWebView2AcceleratorKeyPressedEventArgs>) -> Result<(), Error> {
    if let Some(args) = args {
        let args2: ICoreWebView2AcceleratorKeyPressedEventArgs2 = args.cast()?;
        unsafe { args2.SetIsBrowserAcceleratorKeyEnabled(BOOL(0))? };
    }
    Ok(())
}

pub fn change_theme(window: &tauri::WebviewWindow, theme: Theme) {
    change_webview_theme(window, theme);
    allow_dark_mode_for_app(theme);
    unsafe { PostMessageW(window.hwnd().unwrap(), WM_THEMECHANGED, WPARAM(0), LPARAM(0)).unwrap() };
}

fn change_webview_theme(window: &tauri::WebviewWindow, theme: Theme) {
    let webview_theme = match theme {
        Theme::dark => COREWEBVIEW2_PREFERRED_COLOR_SCHEME_DARK,
        Theme::light => COREWEBVIEW2_PREFERRED_COLOR_SCHEME_LIGHT,
    };

    let windows = window.app_handle().webview_windows();

    windows.iter().enumerate().for_each(|(_, (_, win))| {
        win.with_webview(move |webview| unsafe {
            let core_webview_13: ICoreWebView2_13 = webview.controller().CoreWebView2().unwrap().cast().unwrap();
            core_webview_13.Profile().unwrap().SetPreferredColorScheme(webview_theme).unwrap();
        })
        .unwrap();
    })
}

fn allow_dark_mode_for_app(theme: Theme) {
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

        GetProcAddress(*HUXTHEME, PCSTR::from_raw(UXTHEME_SETPREFERREDAPPMODE_ORDINAL as usize as *mut _)).map(|handle| std::mem::transmute(handle))
    });

    if let Some(_set_preferred_app_mode) = *SET_PREFERRED_APP_MODE {
        unsafe {
            _set_preferred_app_mode(if theme == Theme::dark {
                PreferredAppMode::ForceDark
            } else {
                PreferredAppMode::ForceLight
            })
        };
    }
}
