use nonstd::process::{Output, SpawnOption};
use std::path::PathBuf;
use tauri::Manager;

pub async fn spawn(app: &tauri::AppHandle, option: SpawnOption) -> Result<Output, Output> {
    let mut modified_option = option.clone();
    let command_name = PathBuf::from(option.program).components().last().unwrap().as_os_str().to_string_lossy().into_owned();
    let command_path = relative_command_path(app, command_name).unwrap();
    modified_option.program = command_path.to_string_lossy().to_string();
    nonstd::process::spawn(modified_option).await
}

pub fn kill(cancellation_token: String) -> Result<(), String> {
    nonstd::process::kill(cancellation_token)
}

pub fn clear() {
    nonstd::process::clear();
}

fn relative_command_path(app: &tauri::AppHandle, command: String) -> Result<PathBuf, String> {
    match tauri::process::current_binary(&app.env()).map_err(|e| e.to_string())?.parent() {
        #[cfg(windows)]
        Some(exe_dir) => Ok(exe_dir.join(command).with_extension("exe")),
        #[cfg(not(windows))]
        Some(exe_dir) => Ok(exe_dir.join(command)),
        None => Err("CurrentExeHasNoParent".to_string()),
    }
}

pub fn reveal(file_path: String) -> Result<(), String> {
    nonstd::shell::show_item_in_folder(file_path)
}
