use serde::Deserialize;
use serde::Serialize;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
use std::env;
use tauri::Emitter;
use tauri::Manager;
pub mod helper;
pub mod settings;

static PLAYER: &str = "Player";
static PLAY_LIST: &str = "Playlist";
static THEME_DARK: &str = "Dark";

#[derive(Clone, Serialize)]
struct ReadyEvent {
    settings: settings::Settings,
}

#[derive(Clone, Serialize)]
struct LoadFileEvent {
    files: Vec<String>,
}

#[derive(serde::Serialize)]
struct OpenedUrls(Vec<String>);

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
fn get_init_args(app_handle: tauri::AppHandle) -> Vec<String> {
    if let Some(urls) = app_handle.try_state::<OpenedUrls>() {
        return urls.inner().0.clone();
    }

    Vec::new()
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
        tauri::Theme::Dark
    } else {
        tauri::Theme::Light
    };
    window.set_theme(Some(theme)).unwrap();
    helper::change_theme(theme);
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
fn retrieve_settings(window: tauri::WebviewWindow) {
    let app = window.app_handle();
    if let Some(settings) = app.try_state::<settings::Settings>() {
        app.emit(
            "ready",
            ReadyEvent {
                settings: settings.inner().clone(),
            },
        )
        .unwrap();
    }
}

#[tauri::command]
async fn refresh_tag_contextmenu(payload: Vec<String>) {
    helper::refresh_tag_contextmenu(PLAY_LIST, payload).await;
}

fn prepare_windows(app: &tauri::App) -> tauri::Result<()> {
    let settings = app.state::<settings::Settings>().inner();

    let player = app.get_webview_window(PLAYER).unwrap();
    let playlist = app.get_webview_window(PLAY_LIST).unwrap();

    let theme = match settings.theme {
        settings::Theme::Dark => tauri::Theme::Dark,
        settings::Theme::Light => tauri::Theme::Light,
    };

    player.set_theme(Some(theme))?;

    player.set_position(tauri::PhysicalPosition {
        x: settings.bounds.x,
        y: settings.bounds.y,
    })?;

    player.set_size(tauri::PhysicalSize {
        width: settings.bounds.width,
        height: settings.bounds.height,
    })?;

    player.show()?;

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

    if settings.playlistVisible {
        playlist.show()?;
    }

    helper::create_playlist_menu(&playlist, settings)?;

    helper::create_sort_menu(&playlist, settings)?;

    app.emit(
        "ready",
        ReadyEvent {
            settings: settings.clone(),
        },
    )?;

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

            let mut urls = Vec::new();
            for arg in env::args().skip(1) {
                urls.push(arg);
            }

            app.manage(OpenedUrls(urls));

            Ok(())
        })
        .on_window_event(|win, ev| {
            if let tauri::WindowEvent::Resized(_) = ev {
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
        })
        .invoke_handler(tauri::generate_handler![get_init_args, retrieve_settings, save, change_theme, open_context_menu, open_sort_context_menu, refresh_tag_contextmenu])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
