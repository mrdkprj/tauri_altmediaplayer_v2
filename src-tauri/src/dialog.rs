use serde::{Deserialize, Serialize};
use zouni::dialog::{message, FileDialogResult, FileFilter, MessageDialogKind, MessageDialogOptions, OpenDialogOptions, OpenProperty, SaveDialogOptions};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogOptions {
    dialog_type: String,
    title: Option<String>,
    kind: Option<String>,
    cancel_id: Option<u32>,
    buttons: Option<Vec<String>>,
    message: String,
}

pub async fn show(options: DialogOptions) -> bool {
    match options.dialog_type.as_str() {
        "message" => show_message(options).await,
        "confirm" => show_confirm(options).await,
        "ask" => show_ask(options).await,
        _ => false,
    }
}

fn get_level(kind: &Option<String>) -> MessageDialogKind {
    if let Some(kind) = kind {
        match kind.as_str() {
            "options" => MessageDialogKind::Info,
            "warning" => MessageDialogKind::Warning,
            "error" => MessageDialogKind::Error,
            _ => MessageDialogKind::Info,
        }
    } else {
        MessageDialogKind::Info
    }
}

async fn show_message(options: DialogOptions) -> bool {
    let options = MessageDialogOptions {
        title: options.title,
        kind: Some(get_level(&options.kind)),
        buttons: options.buttons.unwrap_or_default(),
        message: options.message,
        cancel_id: options.cancel_id,
    };
    message(options).await
}

async fn show_confirm(options: DialogOptions) -> bool {
    let options = MessageDialogOptions {
        title: options.title,
        kind: Some(get_level(&options.kind)),
        buttons: options.buttons.unwrap_or(vec!["OK".to_string(), "Cancel".to_string()]),
        message: options.message,
        cancel_id: options.cancel_id,
    };
    message(options).await
}

async fn show_ask(options: DialogOptions) -> bool {
    let options = MessageDialogOptions {
        title: options.title,
        kind: Some(get_level(&options.kind)),
        buttons: options.buttons.unwrap_or(vec!["Yes".to_string(), "No".to_string()]),
        message: options.message,
        cancel_id: options.cancel_id,
    };
    message(options).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDialogOptions {
    pub title: Option<String>,
    pub default_path: Option<String>,
    pub filters: Option<Vec<FileFilter>>,
    pub properties: Option<Vec<OpenProperty>>,
}

pub async fn open(options: FileDialogOptions) -> FileDialogResult {
    let options = OpenDialogOptions {
        title: options.title,
        default_path: options.default_path,
        filters: options.filters,
        properties: options.properties,
    };

    zouni::dialog::open(options).await
}

pub async fn save(options: FileDialogOptions) -> FileDialogResult {
    let options = SaveDialogOptions {
        title: options.title,
        default_path: options.default_path,
        filters: options.filters,
    };

    zouni::dialog::save(options).await
}
