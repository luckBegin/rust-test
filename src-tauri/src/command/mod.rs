use tauri::async_runtime::handle;
use tauri::window;
use tauri::Manager;
use tauri::Emitter;
use crate::GLOBAL_APP_HANDLE;
use crate::util::km_detect::{detect, HidInfo};

#[tauri::command]
pub async fn devices() -> Result<Vec<HidInfo>, ()> {
    let devices = detect().await;
    Ok(devices)
}
