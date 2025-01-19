use nonstd::ClipboardData;
use nonstd::FileAttribute;
use nonstd::Operation;
use serde::Deserialize;
use serde::Serialize;
use settings::Settings;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
use std::env;
use std::path::PathBuf;
use tauri::Emitter;
use tauri::Manager;
use tauri::WebviewWindow;
pub mod helper;
pub mod settings;

static PLAYER: &str = "Player";
static PLAY_LIST: &str = "Playlist";
static THEME_DARK: &str = "Dark";

fn get_window_handel(window: &WebviewWindow) -> isize {
    window.hwnd().unwrap().0 as _
}

#[derive(Clone, Serialize)]
struct LoadFileEvent {
    files: Vec<String>,
}

#[derive(serde::Serialize)]
struct OpenedUrls(Vec<String>);

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
fn reveal(payload: String) -> Result<(), String> {
    nonstd::shell::show_item_in_folder(payload)
}

#[tauri::command]
fn trash(payload: String) -> Result<(), String> {
    nonstd::shell::trash(payload)
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
    nonstd::fs::get_file_attribute(&payload)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopyInfo {
    from: String,
    to: String,
}
#[tauri::command]
fn copy_file(payload: CopyInfo) -> Result<u64, String> {
    std::fs::copy(payload.from, payload.to).map_err(|e| e.to_string())
}

#[tauri::command]
fn mv(payload: CopyInfo) -> Result<(), String> {
    nonstd::fs::mv(payload.from, payload.to, None, None)
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

#[allow(deprecated)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
            exists,
            rename,
            stat,
            copy_file,
            mv,
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
            write_all
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
