use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter,Display};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Bounds {
    pub width:u32,
    pub height:u32,
    pub x:i32,
    pub y:i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Display, EnumIter)]
pub enum SortOrder {
    NameAsc,
    NameDesc,
    DateAsc,
    DateDesc,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Sort {
    pub order: SortOrder,
    pub groupBy:bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Video {
    pub playbackSpeed:f64,
    pub seekSpeed:f64,
    pub fitToWindow: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Audio {
    pub volume:f64,
    pub ampLevel:f64,
    pub mute:bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Display, EnumIter)]
#[allow(non_camel_case_types)]
pub enum Mode {
    system,
    en,
    ja,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Display, EnumIter)]
#[allow(non_camel_case_types)]
pub enum Lang {
    en,
    ja,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Locale {
    pub mode:Mode,
    pub lang:Lang,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Display, EnumIter)]
#[allow(non_camel_case_types)]
pub enum Theme {
    dark = 1,
    light = 2,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Settings {
    pub bounds: Bounds,
    pub playlistBounds: Bounds,
    pub isMaximized: bool,
    pub playlistVisible:bool,
    pub theme:Theme,
    pub sort:Sort,
    pub video:Video,
    pub audio:Audio,
    pub defaultPath:String,
    pub locale:Locale,
    pub tags:Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bounds: Bounds{x:0, y:0, width:1200, height:800},
            playlistBounds: Bounds{x:0, y:0, width:400, height:700},
            isMaximized: false,
            playlistVisible: true,
            theme: Theme::dark,
            sort: Sort{order: SortOrder::NameAsc, groupBy:false},
            video: Video{playbackSpeed:1.0, seekSpeed:5.0, fitToWindow:true},
            audio: Audio{volume:1.0, ampLevel:0.07, mute:false },
            defaultPath: "".to_string(),
            locale: Locale{mode:Mode::system, lang:Lang::en},
            tags:Vec::new()
        }
    }
}

const SETTING_FILE_NAME:&str = "altmediaplayer.settings.json";

pub fn get_settings(app_dir:PathBuf) -> std::io::Result<Settings>{

    let temp_path = Path::new(&app_dir).join("temp");
    if !temp_path.exists() {
        fs::create_dir_all(&temp_path)?;
    }

    let file = temp_path.join(SETTING_FILE_NAME);

    if file.exists() {
        let raw_data = fs::read_to_string(file)?;
        let settings = serde_json::from_str(&raw_data)?;
        Ok(settings)
    } else {
        Ok(Settings::default())
    }
}

pub fn save_settings(app_dir:PathBuf, settings:&mut Settings, player:&tauri::WebviewWindow, list:&tauri::WebviewWindow) -> tauri::Result<bool> {
    settings.bounds.height = player.outer_size()?.height;
    settings.bounds.width = player.outer_size()?.width;
    settings.bounds.x = player.outer_position()?.x;
    settings.bounds.y = player.outer_position()?.y;
    settings.playlistBounds.height = list.outer_size()?.height;
    settings.playlistBounds.width = list.outer_size()?.width;
    settings.playlistBounds.x = list.outer_position()?.x;
    settings.playlistBounds.y = list.outer_position()?.y;

    let file = Path::new(&app_dir).join("temp").join(SETTING_FILE_NAME);
    let data = serde_json::to_string::<Settings>(&settings).unwrap();
    match fs::write(file, data) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false)
    }
}