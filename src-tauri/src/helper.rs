use crate::{Sort, PLAYER};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

pub struct Urls {
    pub taken: bool,
    pub urls: Vec<String>,
}

impl Urls {
    pub fn new(urls: Vec<String>) -> Self {
        Self {
            taken: false,
            urls,
        }
    }

    pub fn add(&mut self, urls: Vec<String>) {
        self.urls.extend(urls);
    }

    pub fn take(&mut self) -> Vec<String> {
        self.taken = true;
        let urls = self.urls.clone();
        self.urls.clear();
        urls
    }
}

pub fn setup(app: &tauri::App) {
    let mut urls = Vec::new();
    for arg in std::env::args().skip(1) {
        urls.push(arg);
    }
    app.manage(Mutex::new(Urls::new(urls)));
}

pub fn handle_second_instance(app: &AppHandle, argv: Vec<String>, _cwd: String) {
    let args = argv[1..].to_vec();
    let state = app.state::<Mutex<Urls>>();
    let mut urls = state.lock().unwrap();
    if urls.taken {
        if let Some(view) = app.get_webview_window(PLAYER) {
            let _ = view.emit("second-instance", args);
        }
    } else {
        urls.add(args);
    }
}

pub fn get_init_args(app: &AppHandle) -> Vec<String> {
    let state = app.state::<Mutex<Urls>>();
    let mut urls = state.lock().unwrap();
    urls.take()
}

pub fn set_sort(app: &AppHandle, new_sort: Sort) {
    if let Some(sort) = app.try_state::<Mutex<Sort>>() {
        *sort.lock().unwrap() = new_sort;
    } else {
        app.manage(Mutex::new(new_sort));
    }
}

pub fn get_sort(app: &AppHandle) -> Sort {
    if let Some(sort) = app.try_state::<Mutex<Sort>>() {
        sort.lock().unwrap().clone()
    } else {
        Sort::default()
    }
}
