//use crate::session::Session;
use tauri::Manager;

pub fn setup(app: &tauri::App) {
    let mut urls = Vec::new();
    for arg in std::env::args().skip(1) {
        urls.push(arg);
    }
    app.manage(urls);

    // let id = &app.config().identifier;
    // if let Ok(session) = crate::session::start(id) {
    //     app.manage(session);
    // }
}

// pub fn exit(app: &tauri::AppHandle) {
//     if let Some(session) = app.try_state::<Session>() {
//         crate::session::end(session.inner());
//     }
// }
