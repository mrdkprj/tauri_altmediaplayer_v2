
use crate::menu::*;
use crate::settings::{Settings, Theme, SortOrder};
use serde::Deserialize;
use tauri::Manager;
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use strum_macros::Display;
use strum::IntoEnumIterator;

static MENU_MAP:Lazy<Mutex<HashMap<String, Menu>>> = Lazy::new(|| Mutex::new(HashMap::new()));

const TAURI_EVENT_NAME:&str = "contextmenu-event";
pub const SORT_MENU_NAME:&str = "Sort";
const PLAYBACK_SPEEDS:[f64;8] = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];
const SEEK_SPEEDS:[f64;9] = [0.03, 0.05, 0.1, 0.5, 1.0, 3.0, 5.0, 10.0, 20.0];
const TEST:bool = false;

#[derive(Debug, Clone, Deserialize)]
pub struct Position {
    x:i32,
    y:i32,
}

#[derive(Clone, Display)]
pub enum PlayerMenu {
    PlaybackSpeed,SeekSpeed,TogglePlaylistWindow,FitToWindow,ToggleFullscreen,Theme,Capture,PictureInPicture
}

#[derive(Clone, Display)]
pub enum PlaylistMenu {
    Remove,RemoveAll,Trash,CopyFileName,CopyFullpath,Reveal,Metadata,Convert,Tag,ManageTags,Sort,Rename,LoadList,SaveList
}

#[derive(Clone, Display)]
pub enum SortMenu {
    GroupBy,NameAsc,NameDesc,DateAsc,DateDesc
}

pub fn popup_menu(app:&tauri::AppHandle, label:&str, position:Position){

    let map = MENU_MAP.lock().unwrap();
    let menu = (*map).get(label).unwrap();

    let result = menu.popup_at(position.x, position.y);

    if result.is_some() {
        app.emit_to(tauri::EventTarget::WebviewWindow { label: label.to_string() }, TAURI_EVENT_NAME, result.unwrap()).unwrap();
    }
}

pub fn create_player_menu(window:&tauri::WebviewWindow, settings:&Settings) -> tauri::Result<()> {

    if TEST {
        return Ok(());
    }
    let hwnd = window.hwnd().unwrap();
    let mut menu = Menu::new_with_theme(hwnd, settings.theme);

    create_playback_speed_submenu(&mut menu, settings);
    create_seek_speed_submenu(&mut menu, settings);
    menu.check(&PlayerMenu::FitToWindow.to_string(), "Fit To Window", "", settings.video.fitToWindow);
    menu.separator();
    menu.text(&PlayerMenu::TogglePlaylistWindow.to_string(), "Playlist\tCtrl+P");
    menu.text(&PlayerMenu::ToggleFullscreen.to_string(), "Toggle Fullscreen\tF11");
    menu.text(&PlayerMenu::PictureInPicture.to_string(), "Picture In Picture");
    menu.separator();
    menu.text(&PlayerMenu::Capture.to_string(), "Capture\tCtrl+S");
    menu.separator();
    create_theme_submenu(&mut menu, settings);

    menu.build_and_attach().unwrap();

    let mut map = MENU_MAP.lock().unwrap();
    (*map).insert(window.label().to_string(), menu);

    Ok(())
}

fn create_playback_speed_submenu(menu:&mut Menu, settings:&Settings){

    let id = PlayerMenu::PlaybackSpeed.to_string();
    let parent = menu.submenu("Playback Speed");

    for (_, speed) in PLAYBACK_SPEEDS.iter().enumerate() {
        let speed_str = &speed.to_string();
        parent.radio(&id, speed_str, speed_str, &id, speed == &settings.video.playbackSpeed);
    };
}

fn create_seek_speed_submenu(menu:&mut Menu, settings:&Settings){

    let id = PlayerMenu::SeekSpeed.to_string();
    let parent = menu.submenu("Seek Speed");

    for (_, speed) in SEEK_SPEEDS.iter().enumerate() {
        let speed_str = &speed.to_string();
        parent.radio(&id, speed_str, speed_str, &id, speed == &settings.video.seekSpeed);
    }
}

fn create_theme_submenu(menu:&mut Menu, settings:&Settings){

    let id = PlayerMenu::Theme.to_string();
    let parent = menu.submenu("Theme");

    for (_, theme) in Theme::iter().enumerate() {
        let theme_str = if theme.to_string() == "dark" { "Dark" } else { "Light" };
        parent.radio(&id, theme_str, &theme.to_string(), &id, theme == settings.theme);
    }
}

pub fn create_playlist_menu(window:&tauri::WebviewWindow, settings:&Settings) -> tauri::Result<()> {

    if TEST {
        return Ok(());
    }

    let hwnd = window.hwnd().unwrap();
    let menu = Menu::new_with_theme(hwnd, settings.theme);

    menu.text(&PlaylistMenu::Remove.to_string(), "Remove\tDelete");
    menu.text(&PlaylistMenu::Trash.to_string(), "Trash\tShift+Delete");
    menu.separator();
    menu.text(&PlaylistMenu::CopyFileName.to_string(), "Copy Name\tCtrl+C");
    menu.text(&PlaylistMenu::CopyFullpath.to_string(), "Copy Full Path\tCtrl+Shift+C");
    menu.text(&PlaylistMenu::Reveal.to_string(), "Reveal in File Explorer\tCtrl+R");
    menu.separator();
    menu.text(&PlaylistMenu::Rename.to_string(), "Rename\tF2");
    menu.text(&PlaylistMenu::Metadata.to_string(), "View Metadata");
    menu.text(&PlaylistMenu::Convert.to_string(), "Convert");
    menu.separator();
    menu.text(&PlaylistMenu::Tag.to_string(), "Add Tag to Comment");
    menu.text(&PlaylistMenu::ManageTags.to_string(), "Manage Tags");
    menu.separator();
    menu.text(&PlaylistMenu::LoadList.to_string(), "Load Playlist");
    menu.text(&PlaylistMenu::SaveList.to_string(), "Save Playlist");
    menu.separator();
    menu.text(&PlaylistMenu::RemoveAll.to_string(), "Clear Playlist");

    menu.build_and_attach().unwrap();

    let mut map = MENU_MAP.lock().unwrap();
    (*map).insert(window.label().to_string(), menu);

    Ok(())
}

pub fn create_sort_menu(window:&tauri::WebviewWindow, settings:&Settings) -> tauri::Result<()> {

    if TEST {
        return Ok(());
    }

    let hwnd = window.hwnd().unwrap();
    let menu = Menu::new_with_theme(hwnd, settings.theme);
    let id = &PlaylistMenu::Sort.to_string();

    menu.check(id, "Group By Directory", &SortMenu::GroupBy.to_string(), settings.sort.groupBy);
    menu.separator();
    menu.radio(id, "Name(Asc)", &SortMenu::NameAsc.to_string(), id, settings.sort.order == SortOrder::NameAsc);
    menu.radio(id, "Name(Desc)", &SortMenu::NameDesc.to_string(), id, settings.sort.order == SortOrder::NameDesc);
    menu.radio(id, "Date(Asc)", &SortMenu::DateAsc.to_string(), id, settings.sort.order == SortOrder::DateAsc);
    menu.radio(id, "Date(Desc", &SortMenu::DateDesc.to_string(), id, settings.sort.order == SortOrder::DateDesc);

    menu.build().unwrap();

    let mut map = MENU_MAP.lock().unwrap();
    (*map).insert(SORT_MENU_NAME.to_string(), menu);

    Ok(())
}