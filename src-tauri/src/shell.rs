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

pub fn reveal(file_path: String, use_file_manager: bool) -> Result<(), String> {
    if use_file_manager {
        nonstd::shell::show_item_in_folder(file_path)
    } else {
        let output = std::process::Command::new("powershell")
            .arg("-Command")
            .arg("Get-ItemProperty HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* | Select-Object DisplayName, InstallLocation, MainBinaryName | ConvertTo-Json")
            .output()
            .unwrap();

        let data: serde_json::Value = serde_json::from_str(std::str::from_utf8(&output.stdout).unwrap()).map_err(|e| e.to_string())?;

        let target: Vec<&serde_json::Value> = data.as_array().unwrap().iter().filter(|item| item["DisplayName"].as_str().unwrap_or_default() == "explite").collect();
        if target.is_empty() {
            println!("No reveal alternative found");
            return Ok(());
        }
        let file_manager = target.first().unwrap();
        let location = file_manager["InstallLocation"].as_str().unwrap_or_default().replace('\"', "");
        let mut local_app_data = PathBuf::from(location);
        local_app_data.push(file_manager["MainBinaryName"].as_str().unwrap());
        nonstd::shell::open_path_with(file_path, local_app_data)
    }
}
