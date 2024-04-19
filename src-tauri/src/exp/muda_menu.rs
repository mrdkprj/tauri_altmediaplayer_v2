use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder, Menu, Submenu, MenuId, CheckMenuItemBuilder, IsMenuItem, CheckMenuItem};
use tauri::{Manager, Wry};
use serde::Serialize;
use strum::IntoEnumIterator;
use crate::settings::{Settings, SortOrder, Theme};
use strum_macros::Display;
//const LEFT: &'static [&'static str] = &["Hello", "World", "!"];

const PLAYBACK_SPEEDS:[f64;8] = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];
const SEEK_SPEEDS:[f64;9] = [0.03, 0.05, 0.1, 0.5, 1.0, 3.0, 5.0, 10.0, 20.0];
const EXCEPT_ID:&str = "GroupBy";

#[derive(Clone, Display)]
enum PlayerMenu {
    PlaybackSpeed,SeekSpeed,TogglePlaylistWindow,FitToWindow,ToggleFullscreen,Theme,Capture,PictureInPicture
}

#[derive(Clone, Display)]
enum PlayerListMenu {
    Remove,RemoveAll,Trash,CopyFileName,CopyFullpath,Reveal,Metadata,Convert,Tag,ManageTags,Sort,Rename,LoadList,SaveList,GroupBy
}

#[derive(Clone, Display)]
enum Menus {
    Player(PlayerMenu),
    List(PlayerListMenu)
}

enum MenuKind<'a, R: tauri::Runtime> {
    Menu(&'a Menu<R>),
    Submenu(&'a Submenu<R>)
}

#[derive(Clone, Serialize, Debug)]
struct MenuIdExt {
    label:String,
    id:String,
    name:String,
    value:String,
    group:String,
}

impl MenuIdExt {

    const SPLITTER:&'static str = ":";

    pub fn new(label:&str) -> Self {
        Self {
            label:label.to_string(),
            id: String::from(""),
            name: String::from(""),
            value: String::from(""),
            group: String::from(""),
        }
    }

    pub fn with_name(&self, name:Menus) -> Self {

        let name = match name {
            Menus::Player(name) => name.to_string(),
            Menus::List(name) => name.to_string(),
        };

        Self {
            label: self.label.to_string(),
            id: Self::create_id(&self.label, &name, &self.value, &self.group),
            name,
            value: self.value.to_string(),
            group: self.group.to_string(),
        }
    }

    pub fn get(id:&MenuId) -> Self {

        if id.0.contains(Self::SPLITTER) {
            let inner_id = id.0.split(Self::SPLITTER).collect::<Vec<&str>>();
            Self {
                label:inner_id[0].to_string(),
                id: id.0.clone(),
                name:inner_id[1].to_string(),
                value:inner_id[2].to_string(),
                group:inner_id[3..].join(Self::SPLITTER).to_string(),
            }
        } else {
            Self {
                label: "".to_string(),
                id: "".to_string(),
                name: "".to_string(),
                value: "".to_string(),
                group: "".to_string(),
            }
        }
    }

    fn create_id(label:&String, name:&String, value:&String, group:&String) -> String{
        format!("{}:{}:{}:{}", label, name, value, group)
    }

    pub fn id(&self, name:Menus) -> String {
        let name = match name {
            Menus::Player(name) => name.to_string(),
            Menus::List(name) => name.to_string(),
        };
        Self::create_id(&self.label, &name, &self.value, &self.group)
    }

    pub fn id_from(&self, value:&str) -> String {
        let group = format!("{}:{}:{}:{}", self.label, self.name, "", "");
        Self::create_id(&self.label, &self.name, &value.to_string(), &group)
    }

}

fn to_items<R: tauri::Runtime>(items:&Vec<Box<dyn IsMenuItem<R>>>) -> Vec<&dyn IsMenuItem<R>> {
    items.iter().map(|item| item.as_ref() as &dyn IsMenuItem<R>).collect::<Vec<_>>()
}

fn toggle_checked<R: tauri::Runtime>(menu_king:MenuKind<Wry>, checked_menu:&CheckMenuItem<R>){

    let items = match menu_king {
        MenuKind::Menu(menu) => menu.items().unwrap(),
        MenuKind::Submenu(sub) => sub.items().unwrap(),
    };

    items.iter()
    .filter(|item| !MenuIdExt::get(&item.id()).id.is_empty())
    .filter(|item| item.id() != EXCEPT_ID)
    .for_each(|item| {
        if item.id() != checked_menu.id() {
            item.as_check_menuitem().unwrap().set_checked(false).unwrap();
        }
    });

}

fn on_menu_event(window:&tauri::Window, event:tauri::menu::MenuEvent){

    let id_ext = MenuIdExt::get(&event.id);

    if id_ext.label != window.label() {
        return;
    }

    if !id_ext.group.is_empty() {
        let mut menu = window.app_handle().menu().unwrap();

        if menu.get(&event.id).is_none() && menu.get(&id_ext.name).is_none() {
            menu = window.menu().unwrap();
        }

        match menu.get(&event.id) {
            Some(item) => {
                if let Some(checked_item) = item.as_check_menuitem() {
                    toggle_checked(MenuKind::Menu(&menu), checked_item);
                }
            },
            None => {
                if let Some(submenu) = menu.get(&id_ext.group).unwrap().as_submenu() {
                    let item = submenu.get(&event.id).unwrap();
                    if let Some(checked_item) = item.as_check_menuitem() {
                        toggle_checked(MenuKind::Submenu(submenu), checked_item);
                    }
                }
            }
        }
    }

    window.emit_to(tauri::EventTarget::WebviewWindow { label: window.label().to_string() }, "contextmenu-event", id_ext).unwrap();
}

#[allow(non_snake_case)]
pub fn create_player_menu(handle:&tauri::AppHandle, window:&tauri::WebviewWindow, settings:&Settings) -> tauri::Result<()> {

    let menu_id = MenuIdExt::new(window.label());
    let playback_speed = create_playback_speed_submenu(handle, menu_id.clone(), settings)?;
    let seekSpeed = create_seek_speed_submenu(handle, menu_id.clone(), settings)?;
    let fitToWindow = CheckMenuItemBuilder::with_id(menu_id.id(Menus::Player(PlayerMenu::FitToWindow)), "fitToWindow").checked(settings.video.fitToWindow).build(handle)?;
    let playlist = MenuItemBuilder::with_id(menu_id.id(Menus::Player(PlayerMenu::TogglePlaylistWindow)), "playlist").build(handle)?;
    let fullscreen = MenuItemBuilder::with_id(menu_id.id(Menus::Player(PlayerMenu::ToggleFullscreen)), "fullscreen").build(handle)?;
    let pip = MenuItemBuilder::with_id(menu_id.id(Menus::Player(PlayerMenu::PictureInPicture)), "pip").build(handle)?;
    let capture = MenuItemBuilder::with_id(menu_id.id(Menus::Player(PlayerMenu::Capture)), "capture").build(handle)?;
    let theme = create_theme_submenu(handle, menu_id.clone(), settings)?;

    let menu = MenuBuilder::new(handle)
    .item(&playback_speed)
    .item(&seekSpeed)
    .item(&fitToWindow)
    .separator()
    .item(&playlist)
    .item(&fullscreen)
    .item(&pip)
    .separator()
    .item(&capture)
    .separator()
    .item(&theme)
    .build()?;

    window.on_menu_event(on_menu_event);

    window.set_menu(menu)?;

    Ok(())
}

fn create_playback_speed_submenu(handle:&tauri::AppHandle, menu_id:MenuIdExt, settings:&Settings) -> tauri::Result<Submenu<Wry>> {
    let menu_id = menu_id.with_name(Menus::Player(PlayerMenu::PlaybackSpeed));
    let parent = SubmenuBuilder::with_id(handle, &menu_id.id, "Playback Speed");
    let mut menu_items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();
    for speed in PLAYBACK_SPEEDS {
        let speed_str = &speed.to_string();
        let item = CheckMenuItemBuilder::with_id(menu_id.id_from(speed_str), speed_str).checked(settings.video.playbackSpeed == speed).build(handle)?;
        menu_items.push(Box::new(item));
    };
    parent.items(&to_items(&menu_items)).build()
}

fn create_seek_speed_submenu(handle:&tauri::AppHandle, menu_id:MenuIdExt, settings:&Settings) -> tauri::Result<Submenu<Wry>> {
    let menu_id = menu_id.with_name(Menus::Player(PlayerMenu::SeekSpeed));
    let parent = SubmenuBuilder::with_id(handle, &menu_id.id, "Seek Speed");
    let mut menu_items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();
    for speed in SEEK_SPEEDS {
        let speed_str = &speed.to_string();
        let item = CheckMenuItemBuilder::with_id(menu_id.id_from(speed_str), speed_str).checked(settings.video.seekSpeed == speed).build(handle)?;
        menu_items.push(Box::new(item));

    }
    parent.items(&to_items(&menu_items)).build()
}

fn create_theme_submenu(handle:&tauri::AppHandle, menu_id:MenuIdExt, settings:&Settings) -> tauri::Result<Submenu<Wry>> {
    let menu_id = menu_id.with_name(Menus::Player(PlayerMenu::Theme));
    let parent = SubmenuBuilder::with_id(handle, &menu_id.id, "Theme");
    let mut menu_items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();
    for theme in Theme::iter() {
        let theme_str = &theme.to_string();
        let item  = CheckMenuItemBuilder::with_id(menu_id.id_from(theme_str), theme_str).checked(settings.theme == theme).build(handle)?;
        menu_items.push(Box::new(item));
    }
    parent.items(&to_items(&menu_items)).build()
}

#[allow(non_snake_case)]
pub fn create_playlist_menu(handle:&tauri::AppHandle, window:&tauri::WebviewWindow) -> tauri::Result<()> {

    let menu_id = MenuIdExt::new(window.label());
    let remove = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::Remove)), "remove").build(handle)?;
    let trash = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::Trash)), "trash").build(handle)?;
    let copyName = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::CopyFileName)), "copyName").build(handle)?;
    let copyFullpath = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::CopyFullpath)), "copyFullpath").build(handle)?;
    let reveal = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::Reveal)), "reveal").build(handle)?;
    let rename = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::Rename)), "rename").build(handle)?;
    let metadata = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::Metadata)), "metadata").build(handle)?;
    let convert = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::Convert)), "convert").build(handle)?;
    let addTag = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::Tag)), "addTag").build(handle)?;
    let manageTag = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::ManageTags)), "manageTag").build(handle)?;
    let loadList = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::LoadList)), "loadList").build(handle)?;
    let saveList = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::SaveList)), "saveList").build(handle)?;
    let clearList = MenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::RemoveAll)), "clearList").build(handle)?;

    let menu = MenuBuilder::new(handle)
    .item(&remove)
    .item(&trash)
    .separator()
    .item(&copyName)
    .item(&copyFullpath)
    .item(&reveal)
    .separator()
    .item(&rename)
    .item(&metadata)
    .item(&convert)
    .separator()
    .item(&addTag)
    .item(&manageTag)
    .separator()
    .item(&loadList)
    .item(&saveList)
    .separator()
    .item(&clearList)
    .build()?;

    window.on_menu_event(on_menu_event);

    window.set_menu(menu)?;

    Ok(())
}

#[allow(non_snake_case)]
pub fn create_sort_menu(handle:&tauri::AppHandle, window:&tauri::WebviewWindow, settings:&Settings) -> tauri::Result<()> {

    let menu_id = MenuIdExt::new(window.label());
    let groupBy = CheckMenuItemBuilder::with_id(menu_id.id(Menus::List(PlayerListMenu::GroupBy)), "groupBy").checked(settings.sort.groupBy).build(handle)?;

    let menu_id = menu_id.with_name(Menus::List(PlayerListMenu::Sort));
    let mut menu_items: Vec<Box<dyn IsMenuItem<Wry>>> = Vec::new();
    for order in SortOrder::iter() {
        let order_str = &order.to_string();
        let item = CheckMenuItemBuilder::with_id(menu_id.id_from(order_str), order_str).checked(settings.sort.order == order).build(handle)?;
        menu_items.push(Box::new(item));
    }

    let menu = MenuBuilder::new(handle)
    .item(&groupBy)
    .separator()
    .items(&to_items(&menu_items))
    .build()?;

    handle.set_menu(menu)?;

    Ok(())
}
