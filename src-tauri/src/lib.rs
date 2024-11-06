use serde::Deserialize;
use serde::Serialize;
use settings::Settings;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
use std::env;
use tauri::Emitter;
use tauri::Manager;
pub mod helper;
pub mod settings;

static PLAYER: &str = "Player";
static PLAY_LIST: &str = "Playlist";
static THEME_DARK: &str = "Dark";

// #[derive(Clone, Serialize)]
// struct ReadyEvent {
//     settings: settings::Settings,
// }

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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct MoveFileRequest {
    sources: Vec<String>,
    dest: String,
    cancellationId: u32,
}

#[tauri::command]
fn get_init_args(app_handle: tauri::AppHandle) -> Vec<String> {
    if let Some(urls) = app_handle.try_state::<OpenedUrls>() {
        return urls.inner().0.clone();
    }

    Vec::new()
}

#[tauri::command]
fn set_settings(app: tauri::AppHandle, payload: Settings) {
    app.manage(payload);
}

#[tauri::command]
fn get_settings(window: tauri::WebviewWindow) -> Settings {
    let app = window.app_handle();
    if let Some(settings) = app.try_state::<Settings>() {
        settings.inner().clone()
    } else {
        Settings::default()
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
async fn refresh_tag_contextmenu(payload: Vec<String>) {
    helper::refresh_tag_contextmenu(PLAY_LIST, payload).await;
}

#[tauri::command]
fn reveal(payload: String) {
    showfile::show_path_in_file_manager(std::path::Path::new(&payload));
}

#[tauri::command]
fn reserve_cancellable() -> u32 {
    helper::reserve_cancellable()
}

#[tauri::command]
async fn move_files(window: tauri::WebviewWindow, payload: MoveFileRequest) -> Result<(), String> {
    println!("{:?}", payload);
    helper::mv(&window, payload.sources, payload.dest, payload.cancellationId).await
}

#[tauri::command]
fn cancel_move(payload: u32) -> bool {
    helper::cancel_move(payload)
}

#[tauri::command]
fn trash(payload: String) -> Result<(), String> {
    helper::trash(payload)
}

#[tauri::command]
fn prepare_windows(app: tauri::AppHandle, payload: Settings) -> tauri::Result<bool> {
    app.manage(payload);

    let settings = app.state::<Settings>();

    let player = app.get_webview_window(PLAYER).unwrap();
    let playlist = app.get_webview_window(PLAY_LIST).unwrap();

    let theme = match settings.theme {
        wcpopup::config::Theme::Dark => tauri::Theme::Dark,
        wcpopup::config::Theme::Light => tauri::Theme::Light,
        _ => tauri::Theme::Dark,
    };

    player.set_theme(Some(theme))?;

    helper::create_player_menu(&player, &settings)?;

    helper::create_playlist_menu(&playlist, &settings)?;
    helper::register_file_drop(&playlist)?;

    helper::create_sort_menu(&playlist, &settings)?;

    Ok(true)
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
        .invoke_handler(tauri::generate_handler![
            get_init_args,
            prepare_windows,
            get_settings,
            set_settings,
            change_theme,
            open_context_menu,
            open_sort_context_menu,
            refresh_tag_contextmenu,
            reveal,
            trash,
            reserve_cancellable,
            move_files,
            cancel_move
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
