use std::collections::HashMap;

use serde::Deserialize;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
use serde::Serialize;
use win32props::read_all;
//use std::{env, sync::OnceLock};

use tauri::Manager;
pub mod helper;
pub mod settings;
pub mod util;

static PLAYER: &str = "Player";
static PLAY_LIST: &str = "Playlist";
static THEME_DARK: &str = "dark";
//static CELL:OnceLock<Vec<String>> = OnceLock::new();

#[derive(Clone, Serialize)]
struct ReadyEvent {
    settings: settings::Settings,
}

#[derive(Clone, Serialize)]
struct LoadFileEvent {
    files: Vec<String>,
}

#[derive(Clone, Serialize)]
#[allow(non_snake_case)]
struct ResizeEvent {
    isMaximized: bool,
}

#[derive(Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct MetadataRequest {
    fullPath: String,
    format: bool,
}

#[tauri::command]
fn save(app: tauri::AppHandle, mut payload: settings::Settings) -> tauri::Result<bool> {
    let dir = app.path().app_data_dir().unwrap();
    let player = app.get_webview_window(PLAYER).unwrap();
    let list = app.get_webview_window(PLAY_LIST).unwrap();
    match settings::save_settings(dir, &mut payload, &player, &list) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
fn change_theme(window: tauri::WebviewWindow, payload: &str) {
    let theme = if payload == THEME_DARK {
        settings::Theme::dark
    } else {
        settings::Theme::light
    };
    util::change_theme(&window, theme);
}

#[tauri::command]
async fn open_context_menu(window: tauri::WebviewWindow, payload: helper::Position) {
    helper::popup_menu(&window, window.label(), payload).await;
}

#[tauri::command]
async fn open_sort_context_menu(window: tauri::WebviewWindow, payload: helper::Position) {
    helper::popup_menu(&window, helper::SORT_MENU_NAME, payload).await;
}

#[tauri::command]
fn get_media_metadata(payload: MetadataRequest) -> tauri::Result<HashMap<String, String>> {
    match read_all(payload.fullPath, payload.format) {
        Ok(meta) => {
            println!("{:?}", meta.len());
            Ok(meta)
        }
        Err(e) => {
            println!("Metadata failed: {:?}", e);
            Ok(HashMap::new())
        }
    }
}

#[tauri::command]
fn retrieve_settings(window: tauri::WebviewWindow) {
    if let Some(state) = window.app_handle().try_state::<settings::Settings>() {
        window
            .emit_to(
                tauri::EventTarget::webview_window(window.label()),
                "ready",
                ReadyEvent {
                    settings: state.inner().clone(),
                },
            )
            .unwrap();
    }
}

fn prepare_windows(app: &tauri::App) -> tauri::Result<()> {
    let settings = app.state::<settings::Settings>().inner();

    let player = app.get_webview_window(PLAYER).unwrap();
    let playlist = app.get_webview_window(PLAY_LIST).unwrap();

    util::init_webview(&player, settings.theme.clone());

    player.set_position(tauri::PhysicalPosition {
        x: settings.bounds.x,
        y: settings.bounds.y,
    })?;
    player.set_size(tauri::PhysicalSize {
        width: settings.bounds.width,
        height: settings.bounds.height,
    })?;
    if settings.isMaximized {
        player.maximize()?;
    }

    helper::create_player_menu(&player, settings)?;

    playlist.set_position(tauri::PhysicalPosition {
        x: settings.playlistBounds.x,
        y: settings.playlistBounds.y,
    })?;
    playlist.set_size(tauri::PhysicalSize {
        width: settings.playlistBounds.width,
        height: settings.playlistBounds.height,
    })?;
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
            app.emit(
                "second-instance",
                LoadFileEvent {
                    files: argv[1..].to_vec(),
                },
            )
            .unwrap();
        }))
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            let settings = settings::get_settings(dir)?;
            app.manage(settings);
            prepare_windows(app)?;
            Ok(())
        })
        .on_window_event(|win, ev| match ev {
            tauri::WindowEvent::Resized(_) => {
                if win.label() == PLAYER {
                    win.emit_to(
                        tauri::EventTarget::webview_window(win.label()),
                        "after-toggle-maximize",
                        ResizeEvent {
                            isMaximized: win.is_maximized().unwrap(),
                        },
                    )
                    .unwrap();
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![retrieve_settings, save, change_theme, open_context_menu, open_sort_context_menu, get_media_metadata])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
