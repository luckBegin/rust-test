use std::sync::{OnceLock};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle,
};
use crate::util::km_detect::watch_device;

pub mod command;
pub mod util;
pub mod types;
pub mod GLOBAL;
pub mod streaming;
pub mod keyboard_mouse;

pub static GLOBAL_APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::async_runtime::spawn(async { watch_device().await });
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            command::devices,
            command::find_lan_device,
            command::capture::check_if_ffmpeg,
            command::capture::download_ffmpeg,
            command::capture::start_live_server,
            command::capture::start_live_server,
            command::capture::end_live_server,
            command::km_capture::start_km_capture,
            command::km_capture::start_km_udp_server,
        ])
        .setup(init_app)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_i])?;
    let _ = GLOBAL_APP_HANDLE.set(app.handle().clone());
    TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .icon(app.default_window_icon().unwrap().clone())
        .build(app)?;
    Ok(())
}
