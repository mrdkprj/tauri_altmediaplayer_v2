
use serde::Serialize;
#[allow(unused_imports)]
use webview2_com::{take_pwstr, Microsoft::Web::WebView2::Win32::{COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_COMMAND, COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_SUBMENU}};
#[allow(unused_imports)]
use webview2_com::{
    pwstr_from_str,string_from_pcwstr,CustomItemSelectedEventHandler,ContextMenuRequestedEventHandler,CreateCoreWebView2CompositionControllerCompletedHandler,
    Microsoft::Web::WebView2::Win32::*
};
#[allow(unused_imports)]
use windows::Win32::Foundation::{POINT,HWND, RECT, BOOL, TRUE};
#[allow(unused_imports)]
use windows::Win32::UI::WindowsAndMessaging::*;
#[allow(unused_imports)]
use windows::Win32::{Foundation::{LPARAM, WPARAM}, UI::WindowsAndMessaging::{DefWindowProcW, GetMenu, PostMessageW, WM_CONTEXTMENU}};
#[allow(unused_imports)]
use windows_core::{ComInterface, PCWSTR, PWSTR, HRESULT};
#[allow(unused_imports)]
use windows::Win32::System::WinRT::EventRegistrationToken;
#[allow(unused_imports)]
use std::sync::OnceLock;
#[allow(unused_imports)]
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute,DWMWA_USE_IMMERSIVE_DARK_MODE};
#[allow(unused_imports)]
use std::ffi::c_void;
#[allow(unused_imports)]
use std::{collections::HashMap, os::windows::ffi::OsStrExt, sync::Mutex};
use once_cell::sync::Lazy;
use strum::IntoEnumIterator;
use strum_macros::Display;
use crate::{settings::{Settings, Theme}, util::ICoreWebView2Ext};

/*
    window.with_webview( move |webview| {
        unsafe{

            let core_webview = ICoreWebView2Ext(webview.controller().CoreWebView2().unwrap());

            let core_webview_13: ICoreWebView2_11 = core_webview.cast::<ICoreWebView2_11>().unwrap();

            let handler = ContextMenuRequestedEventHandler::create(Box::new(handler2));
            let mut token: EventRegistrationToken = EventRegistrationToken::default();
            core_webview_13.add_ContextMenuRequested(&handler, &mut token).unwrap();

        }
    }).unwrap();
*/
type Callback= Box<dyn Fn(MenuId) + Send>;
static EMITTER:Lazy<Mutex<Callback>> = Lazy::new(|| Mutex::new( Box::new(|_| ())));

static MENUS: Lazy<Mutex<Vec<MenuItem>>> = Lazy::new(|| Mutex::new(Vec::new()));

const PLAYBACK_SPEEDS:[f64;8] = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];
const SEEK_SPEEDS:[f64;9] = [0.03, 0.05, 0.1, 0.5, 1.0, 3.0, 5.0, 10.0, 20.0];

pub enum WindoLabel {
    Player,
    Playlist,
}

#[derive(Clone, Display)]
pub enum PlayerMenu {
    PlaybackSpeed,SeekSpeed,TogglePlaylistWindow,FitToWindow,ToggleFullscreen,Theme,Capture,PictureInPicture
}

#[derive(Clone, Display)]
pub enum PlayerListMenu {
    Remove,RemoveAll,Trash,CopyFileName,CopyFullpath,Reveal,Metadata,Convert,Tag,ManageTags,Sort,Rename,LoadList,SaveList,GroupBy
}

#[derive(Clone, Display)]
pub enum Menus {
    Player(PlayerMenu),
    List(PlayerListMenu)
}

struct MenuItem {
    menu:ICoreWebView2ContextMenuItem,
    id:String,
}

unsafe impl Send for MenuItem {}
impl MenuItem{

    pub fn new(menu:ICoreWebView2ContextMenuItem) -> Self {
        Self {
            menu,
            id:String::new(),
        }
    }

    pub fn with_id(menu:ICoreWebView2ContextMenuItem, id:String) -> Self {
        Self {
            menu,
            id,
        }
    }

    pub fn command_id(&self) -> Result<i32, windows_core::Error> {
        let mut command_id = 0;
        unsafe { self.menu.CommandId(&mut command_id)? };
        Ok(command_id)
    }

    pub fn kind(&self) -> Result<COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND, windows_core::Error> {
        let mut kind = COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_SEPARATOR;
        unsafe { self.menu.Kind(&mut kind)? };
        Ok(kind)
    }

    // pub fn children(&self) -> Result<ICoreWebView2ContextMenuItemCollection, windows_core::Error> {
    //     unsafe { self.menu.Children() }
    // }

    pub fn child_count(&self) -> Result<u32, windows_core::Error> {
        let mut count = 0;
        unsafe { self.menu.Children()?.Count(&mut count)? };
        Ok(count)
    }

    pub fn child_at(&self, index:u32) -> Result<MenuItem, windows_core::Error> {
        let menu = unsafe { self.menu.Children()?.GetValueAtIndex(index)? };
        Ok(MenuItem::new(menu))
    }

    pub fn checked(&self, value:bool) -> Result<(), windows_core::Error> {
        unsafe { self.menu.SetIsChecked(value) }
    }

    pub fn toggle_checked(&self) -> Result<(), windows_core::Error> {
        unsafe {
            let mut checked = TRUE;
            self.menu.IsChecked(&mut checked)?;
            checked = !checked;
            Self::checked(self, checked.as_bool())
        }
    }
}

#[derive(Clone, Serialize, Debug)]
pub struct MenuId {
    pub label:String,
    pub name:String,
    pub value:String,
    parent_id:String,
}

impl MenuId {

    const SPLITTER:&'static str = ":";

    pub fn new(label:&str) -> Self {
        Self {
            label:label.to_string(),
            name: String::from(""),
            value: String::from(""),
            parent_id: String::from(""),
        }
    }

    pub fn set_name(&self, name:Menus) -> Self {
        let name = match name {
            Menus::Player(name) => name.to_string(),
            Menus::List(name) => name.to_string(),
        };

        Self {
            label: self.label.to_string(),
            name,
            value: self.value.to_string(),
            parent_id: self.parent_id.to_string(),
        }
    }

    pub fn set_value(&self, value:&str) -> Self {
        Self {
            label: self.label.to_string(),
            name: self.name.to_string(),
            value:value.to_string(),
            parent_id: Self::to_id(self),
        }
    }

    pub fn to_id(&self) -> String {
        format!("{}:{}:{}:{}", self.label, self.name, self.value, self.parent_id)
    }

    pub fn from_id(id:&str) -> Self {

        let inner_id = id.split(Self::SPLITTER).collect::<Vec<&str>>();
        Self {
            label:inner_id[0].to_string(),
            name:inner_id[1].to_string(),
            value:inner_id[2].to_string(),
            parent_id:inner_id[3..].join(Self::SPLITTER).to_string(),
        }
    }

}

pub fn on_menu_event(cb:Callback ){
    *EMITTER.lock().unwrap() = Box::new(cb);
}

pub fn create_player_menu(window:&tauri::WebviewWindow, settings:&Settings) -> Result<(), tauri::Error> {

    let menu_id = MenuId::new(window.label());
    let settings = settings.clone();

    window.with_webview(move |webview| {
        let webview2 = unsafe { webview.controller().CoreWebView2().unwrap() };
        let core_webview2 = ICoreWebView2Ext(webview2);
        context_menu_requested(&core_webview2).unwrap();
        let env = get_env(&core_webview2).unwrap();
        let playback_speed = create_playback_speed_submenu(&env, menu_id.clone(), &settings).unwrap();
        let seek_speed = create_seek_speed_submenu(&env, menu_id.clone(), &settings).unwrap();
        let fit_to_window = create_check_menuitem(&env, "fitToWindow", menu_id.set_name(Menus::Player(PlayerMenu::FitToWindow)).to_id(), settings.video.fitToWindow).unwrap();
        let playlist = create_text_menuitem(&env, "playlist", menu_id.set_name(Menus::Player(PlayerMenu::TogglePlaylistWindow)).to_id()).unwrap();
        let fullscreen = create_text_menuitem(&env, "fullscreen", menu_id.set_name(Menus::Player(PlayerMenu::ToggleFullscreen)).to_id()).unwrap();
        let pip = create_text_menuitem(&env, "pip", menu_id.set_name(Menus::Player(PlayerMenu::PictureInPicture)).to_id()).unwrap();
        let capture = create_text_menuitem(&env, "capture", menu_id.set_name(Menus::Player(PlayerMenu::Capture)).to_id()).unwrap();
        let theme = create_theme_submenu(&env, menu_id.clone(), &settings).unwrap();
        *MENUS.lock().unwrap() = vec![
            playback_speed,
            seek_speed,
            fit_to_window,
            separator(&env).unwrap(),
            playlist,
            fullscreen,
            pip,
            separator(&env).unwrap(),
            capture,
            separator(&env).unwrap(),
            theme
        ]
    })?;

    Ok(())
}

fn create_playback_speed_submenu(env:&ICoreWebView2Environment9, menu_id:MenuId, settings:&Settings) -> Result<MenuItem, windows_core::Error> {

    let menu_id = menu_id.set_name(Menus::Player(PlayerMenu::PlaybackSpeed));
    let parent = unsafe { env.CreateContextMenuItem(to_pcwstr("Playback Speed"), None, COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_SUBMENU)? };

    for (index, speed) in PLAYBACK_SPEEDS.iter().enumerate() {
        let speed_str = &speed.to_string();
        let item = create_check_menuitem(env, speed_str, menu_id.set_value(speed_str).to_id(), speed == &settings.video.playbackSpeed)?;
        unsafe { parent.Children()?.InsertValueAtIndex(index as u32, &item.menu)? };
    };

    Ok(MenuItem::with_id(parent, menu_id.to_id()))
}

fn create_seek_speed_submenu(env:&ICoreWebView2Environment9, menu_id:MenuId, settings:&Settings) -> Result<MenuItem, windows_core::Error> {

    let menu_id = menu_id.set_name(Menus::Player(PlayerMenu::SeekSpeed));
    let parent = unsafe { env.CreateContextMenuItem(to_pcwstr("Seek Speed"), None, COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_SUBMENU)? };

    for (index, speed) in SEEK_SPEEDS.iter().enumerate() {
        let speed_str = &speed.to_string();
        let item = create_check_menuitem(env, speed_str, menu_id.set_value(speed_str).to_id(), speed == &settings.video.seekSpeed)?;
        unsafe { parent.Children()?.InsertValueAtIndex(index as u32, &item.menu)? };
    }

    Ok(MenuItem::with_id(parent, menu_id.to_id()))
}

fn create_theme_submenu(env:&ICoreWebView2Environment9, menu_id:MenuId, settings:&Settings) -> Result<MenuItem, windows_core::Error> {

    let menu_id = menu_id.set_name(Menus::Player(PlayerMenu::Theme));
    let parent = unsafe { env.CreateContextMenuItem(to_pcwstr("Theme"), None, COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_SUBMENU)? };

    for (index, theme) in Theme::iter().enumerate() {
        let theme_str = &theme.to_string();
        let item = create_check_menuitem(env, theme_str, menu_id.set_value(theme_str).to_id(), theme == settings.theme)?;
        unsafe { parent.Children()?.InsertValueAtIndex(index as u32, &item.menu)? };
    }

    Ok(MenuItem::with_id(parent, menu_id.to_id()))
}

pub fn create_playlist_menu(window:&tauri::WebviewWindow, _settings:&Settings) -> Result<(), tauri::Error> {

    let menu_id = MenuId::new(window.label());

    window.with_webview(move |webview| {
        let webview2 = unsafe { webview.controller().CoreWebView2().unwrap() };
        let core_webview2 = ICoreWebView2Ext(webview2);
        context_menu_requested(&core_webview2).unwrap();
        let env = get_env(&core_webview2).unwrap();

        let remove = create_text_menuitem(&env, "remove    Ctrl+X", menu_id.set_name(Menus::List(PlayerListMenu::Remove)).to_id()).unwrap();
        let trash = create_text_menuitem(&env, "trash", menu_id.set_name(Menus::List(PlayerListMenu::Trash)).to_id()).unwrap();
        let copy_name = create_text_menuitem(&env, "copyName", menu_id.set_name(Menus::List(PlayerListMenu::CopyFileName)).to_id()).unwrap();
        let copy_fullpath = create_text_menuitem(&env, "copyFullpath", menu_id.set_name(Menus::List(PlayerListMenu::CopyFullpath)).to_id()).unwrap();
        let reveal = create_text_menuitem(&env, "reveal", menu_id.set_name(Menus::List(PlayerListMenu::Reveal)).to_id()).unwrap();
        let rename = create_text_menuitem(&env, "rename", menu_id.set_name(Menus::List(PlayerListMenu::Rename)).to_id()).unwrap();
        let metadata = create_text_menuitem(&env, "metadata", menu_id.set_name(Menus::List(PlayerListMenu::Metadata)).to_id()).unwrap();
        let convert = create_text_menuitem(&env, "convert", menu_id.set_name(Menus::List(PlayerListMenu::Convert)).to_id()).unwrap();
        let add_tag = create_text_menuitem(&env, "addTag", menu_id.set_name(Menus::List(PlayerListMenu::Tag)).to_id()).unwrap();
        let manage_tag = create_text_menuitem(&env, "manageTag", menu_id.set_name(Menus::List(PlayerListMenu::ManageTags)).to_id()).unwrap();
        let load_list = create_text_menuitem(&env, "loadList", menu_id.set_name(Menus::List(PlayerListMenu::LoadList)).to_id()).unwrap();
        let save_list = create_text_menuitem(&env, "saveList", menu_id.set_name(Menus::List(PlayerListMenu::SaveList)).to_id()).unwrap();
        let clear_list = create_text_menuitem(&env, "clearList", menu_id.set_name(Menus::List(PlayerListMenu::RemoveAll)).to_id()).unwrap();
        *MENUS.lock().unwrap() = vec![
            remove,
            trash,
            separator(&env).unwrap(),
            copy_name,
            copy_fullpath,
            reveal,
            separator(&env).unwrap(),
            rename,
            metadata,
            convert,
            separator(&env).unwrap(),
            add_tag,
            manage_tag,
            separator(&env).unwrap(),
            load_list,
            save_list,
            separator(&env).unwrap(),
            clear_list,
        ]
    })?;

    Ok(())
}

/* Context menu reqeust handler */
fn context_menu_requested(webview2:&ICoreWebView2Ext) -> Result<(), windows_core::Error>  {

    let core_webview_13: ICoreWebView2_11 = webview2.cast::<ICoreWebView2_11>()?;
    let handler = ContextMenuRequestedEventHandler::create(Box::new(on_contextmenu_requested));

    let mut token: EventRegistrationToken = EventRegistrationToken::default();
    unsafe { core_webview_13.add_ContextMenuRequested(&handler, &mut token) }

}

/* text menu */
fn create_text_menuitem(env:&ICoreWebView2Environment9, label:&str, menu_id:String) -> Result<MenuItem, windows_core::Error>  {
    let item = unsafe { env.CreateContextMenuItem(to_pcwstr(&label), None, COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_COMMAND)? };
    set_event_handler(&item, &menu_id)?;
    Ok(MenuItem::with_id(item, menu_id))
}

/* check menu */
fn create_check_menuitem(env:&ICoreWebView2Environment9, label:&str, menu_id:String, checked:bool) -> Result<MenuItem, windows_core::Error>  {
    let item = unsafe { env.CreateContextMenuItem(to_pcwstr(&label), None, COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_CHECK_BOX)? };
    unsafe { item.SetIsChecked(checked)? };
    set_event_handler(&item, &menu_id)?;
    Ok(MenuItem::with_id(item, menu_id))
}

/* separator */
fn separator(env:&ICoreWebView2Environment9) -> Result<MenuItem, windows_core::Error>  {
    let item = unsafe { env.CreateContextMenuItem(to_pcwstr("separator"), None, COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_SEPARATOR)? };
    Ok(MenuItem::with_id(item, "seperator".to_string()))
}

/* create selected eventhandler */
fn set_event_handler(item:&ICoreWebView2ContextMenuItem, menu_id:&str) -> Result<(), windows_core::Error>{
    let mut token = EventRegistrationToken::default();
    let id = menu_id.to_owned();
    let selected_handler = CustomItemSelectedEventHandler::create(
        Box::new(move |sender: Option<ICoreWebView2ContextMenuItem>, _args: Option<windows_core::IUnknown>| {
            if let Some(item) = sender {
                handle_selected(item, &id).unwrap();
            }
            Ok(())
        })
    );
    unsafe { item.add_CustomItemSelected(&selected_handler, &mut token)? };
    Ok(())
}

/* selected event */
fn handle_selected(menu:ICoreWebView2ContextMenuItem, id:&str) -> Result<(), windows_core::Error>{
    println!("commandid: {:?}", id);
    let id_ext = MenuId::from_id(id);

    let item = MenuItem::new(menu);
    let item_kind = item.kind()?;
    let item_id = item.command_id()?;

    if item_kind == COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_CHECK_BOX {

        if id_ext.parent_id.is_empty() {

            item.toggle_checked()?;

        } else {

            let index = find_menu_index(&id_ext.parent_id).unwrap();
            let parent = &MENUS.lock().unwrap()[index];

            for i in 0..parent.child_count().unwrap() {
                let menu = parent.child_at(i)?;
                if menu.command_id()? != item_id {
                    menu.checked(false)?;
                }
            }

        }

    }

    let cb = EMITTER.lock().unwrap();
    cb(id_ext);

    Ok(())
}

fn find_menu_index(id:&str) -> Option<usize> {
    for (index, item) in MENUS.lock().unwrap().iter().enumerate() {
        if item.id == id.to_string() {
            return Some(index);
        }
    }

    None
}

fn get_env(webview2:&ICoreWebView2Ext) -> Result<ICoreWebView2Environment9, windows_core::Error> {
    let core_webview_13 = webview2.cast::<ICoreWebView2_2>()?;
    let env = unsafe { core_webview_13.Environment()? };
    let env9 = env.cast::<ICoreWebView2Environment9>()?;
    Ok(env9)
}

fn to_pcwstr(str:&str) -> PCWSTR {
    PCWSTR::from_raw(pwstr_from_str(str).as_ptr())
}

fn on_contextmenu_requested(_sender: Option<ICoreWebView2>, args: Option<ICoreWebView2ContextMenuRequestedEventArgs>) -> Result<(), windows_core::Error>{

    if let Some(args) = args{

        unsafe {
            let items = args.MenuItems()?;
            let mut count = 0;
            items.Count(&mut count)?;

            for _ in 0..count {
                items.RemoveValueAtIndex(0)?;
            }

            for menu in MENUS.lock().unwrap().iter().rev() {
                items.InsertValueAtIndex(0, &menu.menu)?;
            }
        }
    }

    Ok(())
}
