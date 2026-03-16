use serde::{Deserialize, Serialize};
use std::sync::Mutex;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
use std::{env, path::PathBuf};
use tauri::{Emitter, Manager, WebviewWindow, WindowEvent};
use zouni::{
    dialog::{FileDialogResult, MessageResult},
    ClipboardData, FileAttribute, Operation,
};
mod dialog;
mod helper;
mod menu;
// mod session;
mod shell;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Default, Clone)]
struct Sort {
    order: String,
    groupBy: bool,
}

#[derive(Serialize)]
struct OpenedUrls(Vec<String>);

static PLAYER: &str = "Player";
static PLAY_LIST: &str = "Playlist";

#[cfg(target_os = "linux")]
fn get_window_handle(window: &WebviewWindow) -> isize {
    use gtk::{ffi::GtkApplicationWindow, glib::translate::ToGlibPtr};

    let ptr: *mut GtkApplicationWindow = window.gtk_window().unwrap().to_glib_none().0;
    ptr as isize
}

#[cfg(target_os = "windows")]
fn get_window_handle(window: &WebviewWindow) -> isize {
    window.hwnd().unwrap().0 as _
}

#[tauri::command]
fn get_init_args(app: tauri::AppHandle) -> Vec<String> {
    if let Some(urls) = app.try_state::<OpenedUrls>() {
        return urls.inner().0.clone();
    }

    Vec::new()
}

#[tauri::command]
fn set_sort(app: tauri::AppHandle, payload: Sort) {
    if let Some(sort) = app.try_state::<Mutex<Sort>>() {
        *sort.lock().unwrap() = payload;
    } else {
        app.manage(Mutex::new(payload));
    }
}

#[tauri::command]
fn get_sort(app: tauri::AppHandle) -> Sort {
    if let Some(sort) = app.try_state::<Mutex<Sort>>() {
        sort.lock().unwrap().clone()
    } else {
        Sort::default()
    }
}

#[tauri::command]
fn change_theme(window: WebviewWindow, payload: String) {
    let (tauri_them, menu_theme) = match payload.as_str() {
        "dark" => (tauri::Theme::Dark, wcpopup::config::Theme::Dark),
        "light" => (tauri::Theme::Light, wcpopup::config::Theme::Light),
        _ => (tauri::Theme::Light, wcpopup::config::Theme::System),
    };
    let _ = window.set_theme(Some(tauri_them));
    menu::change_menu_theme(window.app_handle(), menu_theme);
}

#[tauri::command]
async fn open_context_menu(window: WebviewWindow, payload: menu::Position) {
    menu::popup_menu(window.app_handle(), window.label(), menu::PLAYER, payload).await;
}

#[tauri::command]
async fn open_list_context_menu(window: WebviewWindow, payload: menu::Position) {
    menu::popup_menu(window.app_handle(), window.label(), menu::PLAY_LIST, payload).await;
}

#[tauri::command]
async fn open_sort_context_menu(window: WebviewWindow, payload: menu::Position) {
    menu::popup_menu(window.app_handle(), window.label(), menu::SORT_MENU_NAME, payload).await;
}

#[tauri::command]
fn reveal(payload: String) -> Result<(), String> {
    shell::reveal(payload)
}

#[tauri::command]
fn trash(payload: String) -> Result<(), String> {
    zouni::fs::trash(payload)
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
    zouni::fs::stat(&payload)
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
        let attribute = zouni::fs::stat(&path)?;
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
    zouni::fs::mv_all(&payload.from, payload.to)
}

#[tauri::command]
fn is_uris_available() -> bool {
    zouni::clipboard::is_uris_available()
}

#[tauri::command]
fn read_uris(window: WebviewWindow) -> Result<ClipboardData, String> {
    zouni::clipboard::read_uris(get_window_handle(&window))
}

#[tauri::command]
fn read_text(window: WebviewWindow) -> Result<String, String> {
    zouni::clipboard::read_text(get_window_handle(&window))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct WriteUriInfo {
    fullPaths: Vec<String>,
    operation: Operation,
}

#[tauri::command]
fn write_uris(window: WebviewWindow, payload: WriteUriInfo) -> Result<(), String> {
    zouni::clipboard::write_uris(get_window_handle(&window), &payload.fullPaths, payload.operation)
}

#[tauri::command]
fn write_text(window: WebviewWindow, payload: String) -> Result<(), String> {
    zouni::clipboard::write_text(get_window_handle(&window), payload)
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
    zouni::shell::open_path(payload).map_err(|e| e.to_string())
}

#[tauri::command]
async fn message(payload: dialog::DialogOptions) -> MessageResult {
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
    #[cfg(target_os = "windows")]
    {
        let player = app.get_webview_window(PLAYER).unwrap();
        menu::set_play_thumbs(&app, &player, payload);
    }
}

#[tauri::command]
fn set_pause_thumbs(app: tauri::AppHandle, payload: tauri::ipc::Channel<String>) {
    #[cfg(target_os = "windows")]
    {
        let player = app.get_webview_window(PLAYER).unwrap();
        menu::set_pause_thumbs(&app, &player, payload);
    }
}

#[tauri::command]
async fn spawn(app: tauri::AppHandle, payload: zouni::process::SpawnOption) -> Result<zouni::process::Output, zouni::process::Output> {
    shell::spawn(&app, payload).await
}

#[tauri::command]
async fn kill(payload: String) -> Result<(), String> {
    shell::kill(payload)
}

#[allow(unused_variables)]
#[tauri::command]
fn listen_file_drop(window: WebviewWindow, app: tauri::AppHandle, payload: String) -> tauri::Result<()> {
    #[cfg(target_os = "windows")]
    {
        let label = window.label().to_string();
        window.with_webview(move |webview| {
            zouni::webview2::register_file_drop(unsafe { &webview.controller().CoreWebView2().unwrap() }, Some(payload), move |event| {
                app.emit_to(
                    tauri::EventTarget::WebviewWindow {
                        label: label.to_string(),
                    },
                    "tauri://drag-drop",
                    event,
                )
                .unwrap();
            })
            .unwrap();
        })
    }
    #[cfg(target_os = "linux")]
    {
        Ok(())
    }
}

#[tauri::command]
fn unlisten_file_drop() {
    #[cfg(target_os = "windows")]
    zouni::webview2::clear();
}

#[cfg(target_os = "linux")]
#[tauri::command]
fn undo(window: WebviewWindow) -> Result<(), String> {
    window
        .with_webview(|webview_impl| {
            let webview = webview_impl.inner();
            zouni::webkit::execute_editing_command(&webview, "Undo");
        })
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
#[tauri::command]
fn redo(window: WebviewWindow) -> Result<(), String> {
    window
        .with_webview(|webview_impl| {
            let webview = webview_impl.inner();
            zouni::webkit::execute_editing_command(&webview, "Redo");
        })
        .map_err(|e| e.to_string())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Settings {
    pub theme: String,
    pub fitToWindow: bool,
    pub playbackSpeed: f64,
    pub seekSpeed: f64,
    pub groupBy: bool,
    pub order: String,
}
#[tauri::command]
fn prepare_windows(app: tauri::AppHandle, payload: Settings) -> tauri::Result<bool> {
    let player = app.get_webview_window(PLAYER).unwrap();
    let playlist = app.get_webview_window(PLAY_LIST).unwrap();
    let player_window_handle = get_window_handle(&player);
    let list_window_handle = get_window_handle(&playlist);

    let theme = match payload.theme.as_str() {
        "dark" => tauri::Theme::Dark,
        "light" => tauri::Theme::Light,
        _ => tauri::Theme::Dark,
    };

    player.set_theme(Some(theme))?;

    menu::create(&app, player_window_handle, list_window_handle, &payload);

    Ok(true)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let args = argv[1..].to_vec();
            if let Some(view) = app.get_webview_window(PLAYER) {
                let _ = view.emit("second-instance", args);
            }
        }))
        .setup(|app| {
            helper::setup(app);
            Ok(())
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
            #[cfg(target_os = "linux")]
            undo,
            #[cfg(target_os = "linux")]
            redo,
            get_sort,
            set_sort,
            change_theme,
            open_context_menu,
            open_list_context_menu,
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
            launch,
            listen_file_drop,
            unlisten_file_drop,
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
