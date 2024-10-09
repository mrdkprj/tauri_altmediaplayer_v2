use crate::settings::Theme;

pub fn change_theme(window: &tauri::WebviewWindow, theme: Theme) {
    let tauri_theme = match theme {
        Theme::Dark => tauri::Theme::Dark,
        Theme::Light => tauri::Theme::Light,
    };
    window.set_theme(Some(tauri_theme)).unwrap();
}
