use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Bounds {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
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
    pub groupBy: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Video {
    pub playbackSpeed: f64,
    pub seekSpeed: f64,
    pub fitToWindow: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Audio {
    pub volume: f64,
    pub ampLevel: f64,
    pub mute: bool,
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
    pub mode: Mode,
    pub lang: Lang,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Display, EnumIter)]
pub enum Theme {
    Dark = 1,
    Light = 2,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[allow(non_snake_case)]
pub struct Settings {
    pub bounds: Bounds,
    pub playlistBounds: Bounds,
    pub isMaximized: bool,
    pub playlistVisible: bool,
    pub theme: Theme,
    pub sort: Sort,
    pub video: Video,
    pub audio: Audio,
    pub defaultPath: String,
    pub locale: Locale,
    pub tags: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bounds: Bounds {
                x: 0,
                y: 0,
                width: 1200,
                height: 800,
            },
            playlistBounds: Bounds {
                x: 0,
                y: 0,
                width: 400,
                height: 700,
            },
            isMaximized: false,
            playlistVisible: true,
            theme: Theme::Dark,
            sort: Sort {
                order: SortOrder::NameAsc,
                groupBy: false,
            },
            video: Video {
                playbackSpeed: 1.0,
                seekSpeed: 5.0,
                fitToWindow: true,
            },
            audio: Audio {
                volume: 1.0,
                ampLevel: 0.07,
                mute: false,
            },
            defaultPath: "".to_string(),
            locale: Locale {
                mode: Mode::system,
                lang: Lang::en,
            },
            tags: Vec::new(),
        }
    }
}
