use nonstd::{dialog::FileDialogResult, ClipboardData, FileAttribute, Operation};
use serde::{Deserialize, Serialize};
#[cfg_attr(mobile, tauri::mobile_entry_point)]
use std::{env, path::PathBuf};
use tauri::{Emitter, Manager, WebviewWindow, WindowEvent};
mod dialog;
mod helper;
mod shell;

static PLAYER: &str = "Player";
static PLAY_LIST: &str = "Playlist";
static THEME_DARK: &str = "Dark";

fn get_window_handel(window: &WebviewWindow) -> isize {
    window.hwnd().unwrap().0 as _
}

#[tauri::command]
fn get_init_args(app: tauri::AppHandle) -> Vec<String> {
    if let Some(urls) = app.try_state::<OpenedUrls>() {
        return urls.inner().0.clone();
    }

    Vec::new()
}

#[derive(Serialize)]
struct OpenedUrls(Vec<String>);

#[tauri::command]
fn set_settings(app: tauri::AppHandle, payload: Settings) {
    app.manage(payload);
}

#[tauri::command]
fn update_settings(app: tauri::AppHandle, payload: Settings) {
    app.manage(payload.clone());
    app.emit_to(
        tauri::EventTarget::WebviewWindow {
            label: PLAYER.to_string(),
        },
        "settings-updated",
        payload,
    )
    .unwrap();
}

#[tauri::command]
fn get_settings(window: tauri::WebviewWindow) -> String {
    let app = window.app_handle();
    if let Some(settings) = app.try_state::<Settings>() {
        settings.data.clone()
    } else {
        String::from("{}")
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
    #[cfg(target_os = "windows")]
    {
        helper::popup_menu(&window, window.label(), payload).await;
    }
    #[cfg(target_os = "linux")]
    {
        let gtk_window = window.clone();
        window
            .run_on_main_thread(move || {
                gtk::glib::spawn_future_local(async move {
                    helper::popup_menu(&window, window.label(), payload).await;
                });
            })
            .unwrap();
    }
}

#[tauri::command]
async fn open_sort_context_menu(window: tauri::WebviewWindow, payload: helper::Position) {
    #[cfg(target_os = "windows")]
    {
        helper::popup_menu(&window, helper::SORT_MENU_NAME, payload).await;
    }
    #[cfg(target_os = "linux")]
    {
        let gtk_window = window.clone();
        window
            .run_on_main_thread(move || {
                gtk::glib::spawn_future_local(async move {
                    helper::popup_menu(&window, helper::SORT_MENU_NAME, payload).await;
                });
            })
            .unwrap();
    }
}

#[tauri::command]
fn reveal(payload: String) -> Result<(), String> {
    shell::reveal(payload)
}

#[tauri::command]
fn trash(payload: String) -> Result<(), String> {
    nonstd::fs::trash(payload)
}

#[tauri::command]
fn exists(payload: String) -> bool {
    PathBuf::from(payload).exists()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RenameInfo {
    new: String,
    old: String,
}
#[tauri::command]
fn rename(payload: RenameInfo) -> Result<(), String> {
    std::fs::rename(payload.old, payload.new).map_err(|e| e.to_string())
}

#[tauri::command]
fn stat(payload: String) -> Result<FileAttribute, String> {
    nonstd::fs::stat(&payload)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileAttributeEx {
    full_path: String,
    attribute: FileAttribute,
}
#[tauri::command]
fn stat_all(payload: Vec<String>) -> Result<Vec<FileAttributeEx>, String> {
    let mut result = Vec::new();
    for path in payload {
        let attribute = nonstd::fs::stat(&path)?;
        result.push(FileAttributeEx {
            full_path: path,
            attribute,
        });
    }
    Ok(result)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MoveInfo {
    from: Vec<String>,
    to: String,
}
#[tauri::command]
async fn mv_all(payload: MoveInfo) -> Result<(), String> {
    nonstd::fs::mv_all(&payload.from, payload.to)
}

#[tauri::command]
fn is_uris_available() -> bool {
    nonstd::clipboard::is_uris_available()
}

#[tauri::command]
fn read_uris(window: WebviewWindow) -> Result<ClipboardData, String> {
    nonstd::clipboard::read_uris(get_window_handel(&window))
}

#[tauri::command]
fn read_text(window: WebviewWindow) -> Result<String, String> {
    nonstd::clipboard::read_text(get_window_handel(&window))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct WriteUriInfo {
    fullPaths: Vec<String>,
    operation: Operation,
}

#[tauri::command]
fn write_uris(window: WebviewWindow, payload: WriteUriInfo) -> Result<(), String> {
    nonstd::clipboard::write_uris(get_window_handel(&window), &payload.fullPaths, payload.operation)
}

#[tauri::command]
fn write_text(window: WebviewWindow, payload: String) -> Result<(), String> {
    nonstd::clipboard::write_text(get_window_handel(&window), payload)
}

#[tauri::command]
fn mkdir(payload: String) -> Result<(), String> {
    std::fs::create_dir(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn mkdir_all(payload: String) -> Result<(), String> {
    std::fs::create_dir_all(payload).map_err(|e| e.to_string())
}

#[tauri::command]
fn create(payload: String) -> Result<(), String> {
    match std::fs::File::create(payload) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn read_text_file(payload: String) -> Result<String, String> {
    std::fs::read_to_string(payload).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct WriteFileInfo {
    fullPath: String,
    data: String,
}

#[tauri::command]
fn write_text_file(payload: WriteFileInfo) -> Result<(), String> {
    std::fs::write(payload.fullPath, payload.data).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove(payload: String) -> Result<(), String> {
    std::fs::remove_file(payload).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct WriteAllFileInfo {
    fullPath: String,
    data: Vec<u8>,
}
#[tauri::command]
fn write_all(payload: WriteAllFileInfo) -> Result<(), String> {
    std::fs::write(payload.fullPath, payload.data).map_err(|e| e.to_string())
}

#[tauri::command]
fn launch(payload: String) -> Result<(), String> {
    nonstd::shell::open_path(payload).map_err(|e| e.to_string())
}

#[tauri::command]
async fn message(payload: dialog::DialogOptions) -> bool {
    dialog::show(payload).await
}

#[tauri::command]
async fn open(payload: dialog::FileDialogOptions) -> FileDialogResult {
    dialog::open(payload).await
}

#[tauri::command]
async fn save(payload: dialog::FileDialogOptions) -> FileDialogResult {
    dialog::save(payload).await
}

#[tauri::command]
fn set_play_thumbs(app: tauri::AppHandle, payload: tauri::ipc::Channel<String>) {
    let player = app.get_webview_window(PLAYER).unwrap();
    helper::set_play_thumbs(&app, &player, payload);
}

#[tauri::command]
fn set_pause_thumbs(app: tauri::AppHandle, payload: tauri::ipc::Channel<String>) {
    let player = app.get_webview_window(PLAYER).unwrap();
    helper::set_pause_thumbs(&app, &player, payload);
}

#[tauri::command]
async fn spawn(app: tauri::AppHandle, payload: nonstd::process::SpawnOption) -> Result<nonstd::process::Output, nonstd::process::Output> {
    shell::spawn(&app, payload).await
}

#[tauri::command]
async fn kill(payload: String) -> Result<(), String> {
    shell::kill(payload)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Settings {
    pub data: String,
    pub theme: wcpopup::config::Theme,
    pub fitToWindow: bool,
    pub playbackSpeed: f64,
    pub seekSpeed: f64,
    pub groupBy: bool,
    pub order: String,
    pub playerDropTarget: String,
    pub playlistDropTarget: String,
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

    #[cfg(target_os = "windows")]
    helper::register_file_drop(&player, settings.playerDropTarget.clone())?;
    #[cfg(target_os = "windows")]
    helper::register_file_drop(&playlist, settings.playlistDropTarget.clone())?;

    helper::create_sort_menu(&playlist, &settings)?;

    Ok(true)
}

#[allow(deprecated)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(urls) = app.try_state::<OpenedUrls>() {
                let mut seconds = argv[1..].to_vec();
                seconds.extend(urls.inner().0.clone());
                app.manage(OpenedUrls(seconds));
            }
        }))
        .setup(|app| {
            let mut urls = Vec::new();
            for arg in env::args().skip(1) {
                urls.push(arg);
            }

            app.manage(OpenedUrls(urls));

            Ok(())
        })
        .on_page_load(|window, _| {
            if window.webview_windows().len() == 3 {
                window
                    .emit_to(
                        tauri::EventTarget::WebviewWindow {
                            label: PLAYER.to_string(),
                        },
                        "backend-ready",
                        String::new(),
                    )
                    .unwrap();
            }
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Destroyed = event {
                if window.label() == PLAYER {
                    shell::clear();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_init_args,
            prepare_windows,
            get_settings,
            set_settings,
            update_settings,
            change_theme,
            open_context_menu,
            open_sort_context_menu,
            reveal,
            trash,
            exists,
            rename,
            stat,
            mv_all,
            is_uris_available,
            read_uris,
            read_text,
            write_uris,
            write_text,
            mkdir,
            mkdir_all,
            create,
            read_text_file,
            write_text_file,
            remove,
            write_all,
            stat_all,
            set_play_thumbs,
            set_pause_thumbs,
            message,
            open,
            save,
            spawn,
            kill,
            launch
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
