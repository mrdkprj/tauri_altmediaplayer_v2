use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::Manager;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandOption {
    command: String,
    args: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
    pub success: bool,
    pub code: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Output {
    pub status: Status,
    pub stdout: String,
    pub stderr: String,
}

//static CHILDREN: Lazy<Mutex<HashMap<u32, Child>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn execute(app: &tauri::AppHandle, option: CommandOption) -> Output {
    let command_name = std::path::PathBuf::from(option.command).components().last().unwrap().as_os_str().to_string_lossy().into_owned();
    let command_path = relative_command_path(app, command_name).unwrap();
    let mut command = Command::new(command_path);
    command.args(option.args);

    async_std::task::spawn(async move {
        match command.output() {
            Ok(output) => Output {
                status: Status {
                    success: output.status.success(),
                    code: output.status.code(),
                },
                stderr: String::from_utf8(output.stderr).unwrap_or_default(),
                stdout: String::from_utf8(output.stdout).unwrap_or_default(),
            },
            Err(e) => Output {
                status: Status {
                    success: false,
                    code: e.raw_os_error(),
                },
                stderr: String::new(),
                stdout: String::new(),
            },
        }
    })
    .await
}

fn relative_command_path(app: &tauri::AppHandle, command: String) -> Result<std::path::PathBuf, String> {
    match tauri::process::current_binary(&app.env()).map_err(|e| e.to_string())?.parent() {
        #[cfg(windows)]
        Some(exe_dir) => Ok(exe_dir.join(command).with_extension("exe")),
        #[cfg(not(windows))]
        Some(exe_dir) => Ok(exe_dir.join(command)),
        None => Err("CurrentExeHasNoParent".to_string()),
    }
}
