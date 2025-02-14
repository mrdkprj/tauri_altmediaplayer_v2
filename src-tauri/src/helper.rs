use crate::settings::{Settings, SortOrder};
use async_std::sync::Mutex;
#[cfg(target_os = "windows")]
use nonstd::ThumbButton;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    os::windows::ffi::OsStrExt,
};
use strum_macros::Display;
use tauri::{path::BaseDirectory, Emitter, Manager};
use wcpopup::{
    config::{ColorScheme, Config, MenuSize, Theme as MenuTheme, ThemeColor, DEFAULT_DARK_COLOR_SCHEME},
    Menu, MenuBuilder, MenuItem, MenuItemType,
};
#[cfg(target_os = "windows")]
use webview2_com::{
    Microsoft::Web::WebView2::Win32::{
        ICoreWebView2, ICoreWebView2File, ICoreWebView2WebMessageReceivedEventArgs, ICoreWebView2WebMessageReceivedEventArgs2, ICoreWebView2WebMessageReceivedEventHandler,
    },
    WebMessageReceivedEventHandler,
};
#[cfg(target_os = "windows")]
use windows::{
    core::{Interface, PCWSTR, PWSTR},
    Win32::System::WinRT::EventRegistrationToken,
};

static MENU_MAP: Lazy<Mutex<HashMap<String, Menu>>> = Lazy::new(|| Mutex::new(HashMap::new()));

const MENU_EVENT_NAME: &str = "contextmenu-event";
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
                MENU_EVENT_NAME,
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
            let item = MenuItem::builder(MenuItemType::Text).id(&key).label(&key).build();
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

    let config = get_menu_config(settings.theme);

    let mut builder = MenuBuilder::new_for_hwnd_from_config(hwnd, config);

    create_playback_speed_submenu(&mut builder, settings);
    create_seek_speed_submenu(&mut builder, settings);
    builder.check(&PlayerMenu::FitToWindow.to_string(), "Fit To Window", settings.video.fitToWindow, false);
    builder.separator();
    builder.text_with_accelerator(&PlayerMenu::TogglePlaylistWindow.to_string(), "Playlist", false, "Ctrl+P");
    builder.text_with_accelerator(&PlayerMenu::ToggleFullscreen.to_string(), "Toggle Fullscreen", false, "F11");
    builder.text(&PlayerMenu::PictureInPicture.to_string(), "Picture In Picture", false);
    builder.separator();
    builder.text_with_accelerator(&PlayerMenu::Capture.to_string(), "Capture", false, "Ctrl+S");
    builder.separator();
    create_theme_submenu(&mut builder, settings);

    let menu = builder.build().unwrap();

    let mut map = MENU_MAP.try_lock().unwrap();
    (*map).insert(window.label().to_string(), menu);

    Ok(())
}

fn create_playback_speed_submenu(builder: &mut MenuBuilder, settings: &Settings) {
    let id = PlayerMenu::PlaybackSpeed.to_string();
    let mut parent = builder.submenu(&id, "Playback Speed", false);

    for speed in PLAYBACK_SPEEDS {
        let speed_str = &speed.to_string();
        parent.radio(speed_str, speed_str, &id, speed == settings.video.playbackSpeed, false);
    }

    parent.build().unwrap();
}

fn create_seek_speed_submenu(builder: &mut MenuBuilder, settings: &Settings) {
    let id = PlayerMenu::SeekSpeed.to_string();
    let mut parent = builder.submenu(&id, "Seek Speed", false);

    for speed in SEEK_SPEEDS {
        let speed_str = &speed.to_string();
        parent.radio(speed_str, speed_str, &id, speed == settings.video.seekSpeed, false);
    }

    parent.build().unwrap();
}

fn create_theme_submenu(builder: &mut MenuBuilder, settings: &Settings) {
    let id = PlayerMenu::Theme.to_string();
    let mut parent = builder.submenu(&id, "Theme", false);

    parent.radio("Dark", "Dark", &id, settings.theme == wcpopup::config::Theme::Dark, false);
    parent.radio("Light", "Light", &id, settings.theme == wcpopup::config::Theme::Light, false);

    parent.build().unwrap();
}

pub fn create_playlist_menu(window: &tauri::WebviewWindow, settings: &Settings) -> tauri::Result<()> {
    let hwnd = window.hwnd().unwrap();

    let config = get_menu_config(settings.theme);
    let mut builder = MenuBuilder::new_for_hwnd_from_config(hwnd, config);

    builder.text_with_accelerator(&PlaylistMenu::Remove.to_string(), "Remove", false, "Delete");
    builder.text_with_accelerator(&PlaylistMenu::Trash.to_string(), "Trash", false, "Shift+Delete");
    builder.separator();
    builder.text_with_accelerator(&PlaylistMenu::CopyFileName.to_string(), "Copy Name", false, "Ctrl+C");
    builder.text_with_accelerator(&PlaylistMenu::CopyFullpath.to_string(), "Copy Full Path", false, "Ctrl+Shift+C");
    builder.text_with_accelerator(&PlaylistMenu::Reveal.to_string(), "Reveal in File Explorer", false, "Ctrl+R");
    builder.separator();
    builder.text_with_accelerator(&PlaylistMenu::Rename.to_string(), "Rename", false, "F2");
    builder.text(&PlaylistMenu::Metadata.to_string(), "View Metadata", false);
    builder.text(&PlaylistMenu::Convert.to_string(), "Convert", false);
    builder.separator();
    create_tag_context_menu(&mut builder, settings);
    builder.text(&PlaylistMenu::ManageTags.to_string(), "Manage Tags", false);
    builder.separator();
    builder.text(&PlaylistMenu::Move.to_string(), "Move File", false);
    builder.separator();
    builder.text(&PlaylistMenu::RemoveAll.to_string(), "Clear Playlist", false);

    let menu = builder.build().unwrap();

    let mut map = MENU_MAP.try_lock().unwrap();
    (*map).insert(window.label().to_string(), menu);

    Ok(())
}

fn create_tag_context_menu(builder: &mut MenuBuilder, settings: &Settings) {
    let mut parent = builder.submenu(&PlaylistMenu::Tag.to_string(), "Add Tag to Comment", false);
    for tag in &settings.tags {
        parent.text(tag, tag, false);
    }
    parent.build().unwrap();
}

pub fn create_sort_menu(window: &tauri::WebviewWindow, settings: &Settings) -> tauri::Result<()> {
    let hwnd = window.hwnd().unwrap();
    let config = get_menu_config(settings.theme);
    let mut builder = MenuBuilder::new_for_hwnd_from_config(hwnd, config);

    let id = &PlaylistMenu::Sort.to_string();

    builder.check(&SortMenu::GroupBy.to_string(), "Group By Directory", settings.sort.groupBy, false);
    builder.separator();
    builder.radio(&SortMenu::NameAsc.to_string(), "Name(Asc)", id, settings.sort.order == SortOrder::NameAsc, false);
    builder.radio(&SortMenu::NameDesc.to_string(), "Name(Desc)", id, settings.sort.order == SortOrder::NameDesc, false);
    builder.radio(&SortMenu::DateAsc.to_string(), "Date(Asc)", id, settings.sort.order == SortOrder::DateAsc, false);
    builder.radio(&SortMenu::DateDesc.to_string(), "Date(Desc", id, settings.sort.order == SortOrder::DateDesc, false);

    let menu = builder.build().unwrap();

    let mut map = MENU_MAP.try_lock().unwrap();
    (*map).insert(SORT_MENU_NAME.to_string(), menu);

    Ok(())
}

pub fn set_play_thumbs(app: &tauri::AppHandle, receiver: &tauri::WebviewWindow, ev: tauri::ipc::Channel<String>) {
    #[cfg(target_os = "windows")]
    {
        let buttons = get_thumb_buttons(app, true);
        nonstd::shell::set_thumbar_buttons(receiver.hwnd().unwrap().0 as _, &buttons, move |id| {
            ev.send(id).unwrap();
        })
        .unwrap();
    }
}

pub fn set_pause_thumbs(app: &tauri::AppHandle, receiver: &tauri::WebviewWindow, ev: tauri::ipc::Channel<String>) {
    #[cfg(target_os = "windows")]
    {
        let buttons = get_thumb_buttons(app, false);
        nonstd::shell::set_thumbar_buttons(receiver.hwnd().unwrap().0 as _, &buttons, move |id| {
            ev.send(id).unwrap();
        })
        .unwrap();
    }
}

#[cfg(target_os = "windows")]
fn get_thumb_buttons(app: &tauri::AppHandle, play: bool) -> [ThumbButton; 3] {
    let backward = app.path().resolve("assets/backward.png", BaseDirectory::Resource).unwrap();
    let forward = app.path().resolve("assets/forward.png", BaseDirectory::Resource).unwrap();

    if play {
        let play = app.path().resolve("assets/play.png", BaseDirectory::Resource).unwrap();
        [
            ThumbButton {
                id: String::from("Previous"),
                tool_tip: Some(String::from("Previous")),
                icon: backward,
            },
            ThumbButton {
                id: String::from("Play"),
                tool_tip: Some(String::from("Play")),
                icon: play,
            },
            ThumbButton {
                id: String::from("Next"),
                tool_tip: Some(String::from("Next")),
                icon: forward,
            },
        ]
    } else {
        let pause = app.path().resolve("assets/pause.png", BaseDirectory::Resource).unwrap();
        [
            ThumbButton {
                id: String::from("Previous"),
                tool_tip: Some(String::from("Previous")),
                icon: backward,
            },
            ThumbButton {
                id: String::from("Pause"),
                tool_tip: Some(String::from("Pause")),
                icon: pause,
            },
            ThumbButton {
                id: String::from("Next"),
                tool_tip: Some(String::from("Next")),
                icon: forward,
            },
        ]
    }
}

#[cfg(target_os = "windows")]
pub fn register_file_drop(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    window.with_webview(|webview| {
        let mut token = EventRegistrationToken::default();
        let event_handler: ICoreWebView2WebMessageReceivedEventHandler = WebMessageReceivedEventHandler::create(Box::new(drop_handler));
        unsafe { webview.controller().CoreWebView2().unwrap().add_WebMessageReceived(&event_handler, &mut token).unwrap() };
    })
}

#[derive(Serialize)]
struct File {
    path: String,
    kind: String,
}

#[cfg(target_os = "windows")]
fn drop_handler(webview: Option<ICoreWebView2>, args: Option<ICoreWebView2WebMessageReceivedEventArgs>) -> windows::core::Result<()> {
    unsafe {
        if let Some(args) = args {
            let mut webmessageasstring = PWSTR::null();
            args.TryGetWebMessageAsString(&mut webmessageasstring)?;

            if webmessageasstring.to_string().unwrap() == "getPathForFiles" {
                let args2: ICoreWebView2WebMessageReceivedEventArgs2 = args.cast()?;
                if let Ok(obj) = args2.AdditionalObjects() {
                    let mut count = 0;
                    let mut files = Vec::new();
                    obj.Count(&mut count)?;
                    for i in 0..count {
                        let value = obj.GetValueAtIndex(i)?;
                        if let Ok(file) = value.cast::<ICoreWebView2File>() {
                            let mut path_ptr = PWSTR::null();
                            file.Path(&mut path_ptr)?;
                            let path_str = path_ptr.to_string().unwrap();
                            let full_path = std::path::Path::new(&path_str);
                            files.push(File {
                                kind: if full_path.is_file() {
                                    "file".to_string()
                                } else {
                                    "dir".to_string()
                                },
                                path: full_path.to_string_lossy().to_string(),
                            });
                        }
                    }

                    if files.is_empty() {
                        return Ok(());
                    }

                    if let Ok(str) = serde_json::to_string(&files) {
                        let json = encode_wide(str);
                        if let Some(webview) = webview {
                            webview.PostWebMessageAsJson(PCWSTR::from_raw(json.as_ptr()))?;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(std::iter::once(0)).collect()
}
