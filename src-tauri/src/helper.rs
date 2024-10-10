use crate::settings::{Settings, SortOrder, Theme};
use async_std::sync::Mutex;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;
use strum_macros::Display;
use tauri::Emitter;
use wcpopup::{
    config::{ColorScheme, Config, MenuSize, Theme as MenuTheme, ThemeColor, DEFAULT_DARK_COLOR_SCHEME},
    Menu, MenuBuilder, MenuItem,
};

static MENU_MAP: Lazy<Mutex<HashMap<String, Menu>>> = Lazy::new(|| Mutex::new(HashMap::new()));

const TAURI_EVENT_NAME: &str = "contextmenu-event";
pub const SORT_MENU_NAME: &str = "Sort";
const PLAYBACK_SPEEDS: [f64; 8] = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];
const SEEK_SPEEDS: [f64; 9] = [0.03, 0.05, 0.1, 0.5, 1.0, 3.0, 5.0, 10.0, 20.0];

#[derive(Debug, Clone, Deserialize)]
pub struct Position {
    x: i32,
    y: i32,
}

#[derive(Clone, Display)]
pub enum PlayerMenu {
    PlaybackSpeed,
    SeekSpeed,
    TogglePlaylistWindow,
    FitToWindow,
    ToggleFullscreen,
    Theme,
    Capture,
    PictureInPicture,
}

#[derive(Clone, Display)]
pub enum PlaylistMenu {
    Remove,
    RemoveAll,
    Trash,
    CopyFileName,
    CopyFullpath,
    Reveal,
    Metadata,
    Convert,
    Tag,
    ManageTags,
    Sort,
    Rename,
    Move,
}

#[derive(Clone, Display)]
pub enum SortMenu {
    GroupBy,
    NameAsc,
    NameDesc,
    DateAsc,
    DateDesc,
}

pub async fn popup_menu(window: &tauri::WebviewWindow, menu_name: &str, position: Position) {
    let map = MENU_MAP.lock().await;
    let menu = map.get(menu_name).unwrap();
    let result = menu.popup_at_async(position.x, position.y).await;

    if let Some(item) = result {
        window
            .emit_to(
                tauri::EventTarget::WebviewWindow {
                    label: window.label().to_string(),
                },
                TAURI_EVENT_NAME,
                item,
            )
            .unwrap();
    };
}

pub fn change_theme(theme: tauri::Theme) {
    let map = MENU_MAP.try_lock().unwrap();
    map.values().for_each(|menu| menu.set_theme(from_tauri_theme(theme)));
}

fn from_tauri_theme(theme: tauri::Theme) -> MenuTheme {
    if theme == tauri::Theme::Dark {
        MenuTheme::Dark
    } else {
        MenuTheme::Light
    }
}

fn from_settings_theme(theme: Theme) -> MenuTheme {
    if theme == Theme::Dark {
        MenuTheme::Dark
    } else {
        MenuTheme::Light
    }
}

pub async fn refresh_tag_contextmenu(menu_name: &str, tags: Vec<String>) {
    let map = MENU_MAP.lock().await;
    let menu = map.get(menu_name).unwrap();

    let mut submenu = menu.get_menu_item_by_id(&PlaylistMenu::Tag.to_string()).unwrap().submenu.unwrap();
    let menu_item_map = submenu.items().iter().map(|item| (item.id.clone(), item.clone())).collect::<HashMap<_, _>>();
    let existing_keys = menu_item_map.clone().into_keys().collect::<Vec<_>>();

    let keys: HashSet<String> = [tags.clone(), existing_keys].concat().into_iter().collect();

    for (index, key) in keys.into_iter().enumerate() {
        if menu_item_map.contains_key(&key) && !tags.contains(&key) {
            submenu.remove(menu_item_map.get(&key).unwrap());
        }

        if !menu_item_map.contains_key(&key) && tags.contains(&key) {
            let item = MenuItem::new_text_item(&key, &key, None, None, None);
            submenu.insert(item, index as u32);
        }
    }
}

fn get_menu_config(theme: MenuTheme) -> Config {
    Config {
        theme,
        color: ThemeColor {
            dark: ColorScheme {
                color: 0xefefef,
                background_color: 0x202020,
                ..DEFAULT_DARK_COLOR_SCHEME
            },
            ..Default::default()
        },
        size: MenuSize {
            item_horizontal_padding: 0,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn create_player_menu(window: &tauri::WebviewWindow, settings: &Settings) -> tauri::Result<()> {
    let hwnd = window.hwnd().unwrap();

    let config = get_menu_config(from_settings_theme(settings.theme));

    let mut builder = MenuBuilder::new_for_hwnd_from_config(hwnd, config);

    create_playback_speed_submenu(&mut builder, settings);
    create_seek_speed_submenu(&mut builder, settings);
    builder.check(&PlayerMenu::FitToWindow.to_string(), "Fit To Window", settings.video.fitToWindow, None);
    builder.separator();
    builder.text_with_accelerator(&PlayerMenu::TogglePlaylistWindow.to_string(), "Playlist", None, "Ctrl+P");
    builder.text_with_accelerator(&PlayerMenu::ToggleFullscreen.to_string(), "Toggle Fullscreen", None, "F11");
    builder.text(&PlayerMenu::PictureInPicture.to_string(), "Picture In Picture", None);
    builder.separator();
    builder.text_with_accelerator(&PlayerMenu::Capture.to_string(), "Capture", None, "Ctrl+S");
    builder.separator();
    create_theme_submenu(&mut builder, settings);

    let menu = builder.build().unwrap();

    let mut map = MENU_MAP.try_lock().unwrap();
    (*map).insert(window.label().to_string(), menu);

    Ok(())
}

fn create_playback_speed_submenu(builder: &mut MenuBuilder, settings: &Settings) {
    let id = PlayerMenu::PlaybackSpeed.to_string();
    let mut parent = builder.submenu(&id, "Playback Speed", None);

    for speed in PLAYBACK_SPEEDS {
        let speed_str = &speed.to_string();
        parent.radio(speed_str, speed_str, &id, speed == settings.video.playbackSpeed, None);
    }

    parent.build().unwrap();
}

fn create_seek_speed_submenu(builder: &mut MenuBuilder, settings: &Settings) {
    let id = PlayerMenu::SeekSpeed.to_string();
    let mut parent = builder.submenu(&id, "Seek Speed", None);

    for speed in SEEK_SPEEDS {
        let speed_str = &speed.to_string();
        parent.radio(speed_str, speed_str, &id, speed == settings.video.seekSpeed, None);
    }

    parent.build().unwrap();
}

fn create_theme_submenu(builder: &mut MenuBuilder, settings: &Settings) {
    let id = PlayerMenu::Theme.to_string();
    let mut parent = builder.submenu(&id, "Theme", None);

    for theme in Theme::iter() {
        let theme_str = if theme == Theme::Dark {
            "Dark"
        } else {
            "Light"
        };
        parent.radio(theme_str, theme_str, &id, theme == settings.theme, None);
    }

    parent.build().unwrap();
}

pub fn create_playlist_menu(window: &tauri::WebviewWindow, settings: &Settings) -> tauri::Result<()> {
    let hwnd = window.hwnd().unwrap();

    let config = get_menu_config(from_settings_theme(settings.theme));
    let mut builder = MenuBuilder::new_for_hwnd_from_config(hwnd, config);

    builder.text_with_accelerator(&PlaylistMenu::Remove.to_string(), "Remove", None, "Delete");
    builder.text_with_accelerator(&PlaylistMenu::Trash.to_string(), "Trash", None, "Shift+Delete");
    builder.separator();
    builder.text_with_accelerator(&PlaylistMenu::CopyFileName.to_string(), "Copy Name", None, "Ctrl+C");
    builder.text_with_accelerator(&PlaylistMenu::CopyFullpath.to_string(), "Copy Full Path", None, "Ctrl+Shift+C");
    builder.text_with_accelerator(&PlaylistMenu::Reveal.to_string(), "Reveal in File Explorer", None, "Ctrl+R");
    builder.separator();
    builder.text_with_accelerator(&PlaylistMenu::Rename.to_string(), "Rename", None, "F2");
    builder.text(&PlaylistMenu::Metadata.to_string(), "View Metadata", None);
    builder.text(&PlaylistMenu::Convert.to_string(), "Convert", None);
    builder.separator();
    create_tag_context_menu(&mut builder, settings);
    builder.text(&PlaylistMenu::ManageTags.to_string(), "Manage Tags", None);
    builder.separator();
    builder.text(&PlaylistMenu::Move.to_string(), "Move File", None);
    builder.separator();
    builder.text(&PlaylistMenu::RemoveAll.to_string(), "Clear Playlist", None);

    let menu = builder.build().unwrap();

    let mut map = MENU_MAP.try_lock().unwrap();
    (*map).insert(window.label().to_string(), menu);

    Ok(())
}

fn create_tag_context_menu(builder: &mut MenuBuilder, settings: &Settings) {
    let mut parent = builder.submenu(&PlaylistMenu::Tag.to_string(), "Add Tag to Comment", None);
    for tag in &settings.tags {
        parent.text(tag, tag, None);
    }
    parent.build().unwrap();
}

pub fn create_sort_menu(window: &tauri::WebviewWindow, settings: &Settings) -> tauri::Result<()> {
    let hwnd = window.hwnd().unwrap();
    let config = get_menu_config(from_settings_theme(settings.theme));
    let mut builder = MenuBuilder::new_for_hwnd_from_config(hwnd, config);

    let id = &PlaylistMenu::Sort.to_string();

    builder.check(&SortMenu::GroupBy.to_string(), "Group By Directory", settings.sort.groupBy, None);
    builder.separator();
    builder.radio(&SortMenu::NameDesc.to_string(), "Name(Asc)", id, settings.sort.order == SortOrder::NameAsc, None);
    builder.radio(&SortMenu::NameDesc.to_string(), "Name(Desc)", id, settings.sort.order == SortOrder::NameDesc, None);
    builder.radio(&SortMenu::NameDesc.to_string(), "Date(Asc)", id, settings.sort.order == SortOrder::DateAsc, None);
    builder.radio(&SortMenu::NameDesc.to_string(), "Date(Desc", id, settings.sort.order == SortOrder::DateDesc, None);

    let menu = builder.build().unwrap();

    let mut map = MENU_MAP.try_lock().unwrap();
    (*map).insert(SORT_MENU_NAME.to_string(), menu);

    Ok(())
}
