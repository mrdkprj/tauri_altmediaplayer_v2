#[cfg_attr(mobile, tauri::mobile_entry_point)]

use serde::Serialize;
use std::{env, sync::OnceLock};

use tauri::{webview::PageLoadEvent, Manager};
pub mod settings;
pub mod helper;
pub mod util;
pub mod menu;

static PLAYER:&str = "Player";
static PLAY_LIST:&str = "Playlist";
static THEME_DARK:&str = "dark";
//static CELL:OnceLock<Vec<String>> = OnceLock::new();
static SETTINGS:OnceLock<settings::Settings> = OnceLock::new();

#[derive(Clone, Serialize)]
struct ReadyEvent {
    settings:settings::Settings
}

#[derive(Clone, Serialize)]
struct LoadFileEvent {
    files:Vec<String>
}

#[derive(Clone, Serialize)]
#[allow(non_snake_case)]
struct ResizeEvent {
    isMaximized:bool
}

#[tauri::command]
fn restart(app:tauri::AppHandle){
    app.restart();
}

#[tauri::command]
fn save(app:tauri::AppHandle, payload:settings::Settings) -> tauri::Result<bool> {
    let dir = app.path().app_data_dir().unwrap();
    let player = app.get_webview_window(PLAYER).unwrap();
    let list = app.get_webview_window(PLAY_LIST).unwrap();
    match settings::save_settings(dir, &mut payload.clone(), &player, &list){
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
fn change_theme(_window: tauri::WebviewWindow, payload:&str){
    let theme = if payload == THEME_DARK { settings::Theme::dark } else { settings::Theme::light };
    util::change_theme(&_window, theme);
}

#[tauri::command]
fn open_context_menu(window: tauri::WebviewWindow, payload:helper::Position){
    helper::popup_menu(&window.app_handle(), window.label(), payload);
}

#[tauri::command]
fn open_sort_context_menu(window: tauri::WebviewWindow, payload:helper::Position){
    helper::popup_menu(&window.app_handle(), helper::SORT_MENU_NAME, payload);
}

fn prepare_windows(handle:&tauri::AppHandle) -> tauri::Result<()>{

    let settings = SETTINGS.get().unwrap();

    let player = handle.get_webview_window(PLAYER).unwrap();
    let playlist = handle.get_webview_window(PLAY_LIST).unwrap();

    util::init_apply_theme(&player, settings.theme.clone());

    player.set_position(tauri::PhysicalPosition{x:settings.bounds.x, y:settings.bounds.y})?;
    player.set_size(tauri::PhysicalSize{width:settings.bounds.width, height:settings.bounds.height})?;
    if settings.isMaximized {
        player.maximize()?;
    }

    helper::create_player_menu(&player, settings)?;

    playlist.set_position(tauri::PhysicalPosition{x:settings.playlistBounds.x, y:settings.playlistBounds.y})?;
    playlist.set_size(tauri::PhysicalSize{width:settings.playlistBounds.width, height:settings.playlistBounds.height})?;
    if !settings.playlistVisible {
        playlist.hide()?;
    }

    helper::create_playlist_menu(&playlist, settings)?;

    helper::create_sort_menu(&playlist, settings)?;

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
             app.emit("second-instance", LoadFileEvent { files: argv[1..].to_vec() }).unwrap();
        }))
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            let settings = settings::get_settings(dir)?;
            SETTINGS.set(settings).unwrap();
            prepare_windows(app.app_handle())?;
            Ok(())
        })
        .on_page_load(|win, e| {
            if e.event() == PageLoadEvent::Finished {
                if let Some(settings) = SETTINGS.get() {
                    win.emit_to(tauri::EventTarget::webview_window(win.label()), "ready", ReadyEvent {settings:settings.to_owned()}).unwrap();
                }
            }
        })
        .on_window_event(|win, ev| match ev {

            tauri::WindowEvent::Resized(_) => {
                if win.label() == PLAYER {
                    win.emit_to(tauri::EventTarget::webview_window(win.label()),"after-toggle-maximize", ResizeEvent {isMaximized:win.is_maximized().unwrap()}).unwrap();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![restart, save, change_theme, open_context_menu, open_sort_context_menu])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
