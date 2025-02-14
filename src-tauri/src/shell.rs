use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use shared_child::SharedChild;
use std::{
    collections::HashMap,
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};
use tauri::Manager;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnOption {
    program: String,
    args: Option<Vec<String>>,
    cancellation_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandStatus {
    pub success: bool,
    pub code: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Output {
    pub status: CommandStatus,
    pub stdout: String,
    pub stderr: String,
}

static CHILDREN: Lazy<Mutex<HashMap<String, Arc<SharedChild>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn spawn(app: &tauri::AppHandle, option: SpawnOption) -> Result<Output, Output> {
    let command_name = PathBuf::from(option.program).components().last().unwrap().as_os_str().to_string_lossy().into_owned();
    let command_path = relative_command_path(app, command_name).unwrap();
    let mut command = Command::new(command_path);
    if let Some(args) = option.args {
        command.args(args);
    }
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let token = option.cancellation_token.clone();

    let child = SharedChild::spawn(&mut command).unwrap();
    CHILDREN.lock().unwrap().insert(option.cancellation_token, Arc::new(child));

    async_std::task::spawn(async move {
        let mut children = CHILDREN.lock().unwrap();
        let child = children.get(&token).unwrap();
        match child.wait() {
            Ok(exit_status) => {
                let stdout = if let Some(mut out) = child.take_stdout() {
                    let mut buf = String::new();
                    out.read_to_string(&mut buf).unwrap();
                    buf
                } else {
                    String::new()
                };

                let stderr = if let Some(mut out) = child.take_stderr() {
                    let mut buf = String::new();
                    out.read_to_string(&mut buf).unwrap();
                    buf
                } else {
                    String::new()
                };

                children.remove(&token);

                let result = Output {
                    status: CommandStatus {
                        success: exit_status.success(),
                        code: exit_status.code(),
                    },
                    stderr,
                    stdout,
                };

                if exit_status.success() {
                    Ok(result)
                } else {
                    Err(result)
                }
            }
            Err(e) => Err(Output {
                status: CommandStatus {
                    success: false,
                    code: e.raw_os_error(),
                },
                stderr: String::new(),
                stdout: String::new(),
            }),
        }
    })
    .await
}

pub fn kill(cancellation_token: String) -> Result<(), String> {
    if let Ok(mut children) = CHILDREN.try_lock() {
        if children.contains_key(&cancellation_token) {
            children.get_mut(&cancellation_token).unwrap().kill().map_err(|e| e.to_string())?;
            children.remove(&cancellation_token);
        }
    }

    Ok(())
}

pub fn clear() {
    let children = {
        let mut lock = CHILDREN.lock().unwrap();
        std::mem::take(&mut *lock)
    };
    for child in children.into_values() {
        let _ = child.kill();
    }
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
