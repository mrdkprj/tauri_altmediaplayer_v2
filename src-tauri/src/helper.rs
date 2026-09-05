use crate::PLAYER;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Default)]
struct WindowCount {
    count: u8,
}

const MIN_WINDOW_COUNT: u8 = 3;

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
    app.manage(Mutex::new(WindowCount::default()));
}

pub fn on_page_load(app: &AppHandle) {
    if let Some(state) = app.try_state::<Mutex<WindowCount>>() {
        let mut state = state.lock().unwrap();
        state.count += 1;
        if state.count >= MIN_WINDOW_COUNT {
            let _ = app.emit_to(
                tauri::EventTarget::WebviewWindow {
                    label: PLAYER.to_string(),
                },
                "ready",
                "",
            );
        }
    } else {
        /* on_page_load event can occur before setup */
        app.manage(Mutex::new(WindowCount {
            count: 1,
        }));
    }
}

pub fn get_init_args(app: &AppHandle) -> Vec<String> {
    let state = app.state::<Mutex<Urls>>();
    let mut urls = state.lock().unwrap();
    urls.take()
}
