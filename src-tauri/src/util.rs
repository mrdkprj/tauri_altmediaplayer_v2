use crate::settings::Theme;
use tauri::Manager;
use webview2_com::{
    AcceleratorKeyPressedEventHandler,
    Microsoft::Web::WebView2::Win32::{ICoreWebView2AcceleratorKeyPressedEventArgs, ICoreWebView2AcceleratorKeyPressedEventArgs2, ICoreWebView2Controller, ICoreWebView2_13, COREWEBVIEW2_PREFERRED_COLOR_SCHEME_DARK, COREWEBVIEW2_PREFERRED_COLOR_SCHEME_LIGHT},
};
use windows::{
    core::{Error, Interface},
    Win32::Foundation::BOOL,
};

pub fn init_webview(window: &tauri::WebviewWindow, theme: Theme) {
    for webview_window in window.webview_windows().values() {
        webview_window
            .with_webview(|webview| {
                let mut token = windows::Win32::System::WinRT::EventRegistrationToken::default();
                let handler = AcceleratorKeyPressedEventHandler::create(Box::new(on_accelerator_keypressed));
                unsafe { webview.controller().add_AcceleratorKeyPressed(&handler, &mut token).unwrap() };
            })
            .unwrap();
    }

    change_theme(window, theme);
}

fn on_accelerator_keypressed(_: Option<ICoreWebView2Controller>, args: Option<ICoreWebView2AcceleratorKeyPressedEventArgs>) -> Result<(), Error> {
    if let Some(args) = args {
        let args2: ICoreWebView2AcceleratorKeyPressedEventArgs2 = args.cast()?;
        unsafe { args2.SetIsBrowserAcceleratorKeyEnabled(BOOL(0))? };
    }
    Ok(())
}

pub fn change_theme(window: &tauri::WebviewWindow, theme: Theme) {
    let webview_theme = match theme {
        Theme::Dark => COREWEBVIEW2_PREFERRED_COLOR_SCHEME_DARK,
        Theme::Light => COREWEBVIEW2_PREFERRED_COLOR_SCHEME_LIGHT,
    };

    let windows = window.app_handle().webview_windows();

    windows.iter().for_each(|(_, win)| {
        win.with_webview(move |webview| unsafe {
            let core_webview_13: ICoreWebView2_13 = webview.controller().CoreWebView2().unwrap().cast().unwrap();
            core_webview_13.Profile().unwrap().SetPreferredColorScheme(webview_theme).unwrap();
        })
        .unwrap();
    })
}
