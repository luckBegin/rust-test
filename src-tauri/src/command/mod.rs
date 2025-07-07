use serde::{Deserialize, Serialize};
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

pub enum RustEventType {
    DeviceChange,
}

pub struct RustEvent<T> {
    pub evt_type: RustEventType,
    pub evt_data: T,
}

impl<T> RustEvent<T> {
    pub fn new(evt_type: RustEventType, evt_data: T) -> Self {
        Self { evt_type, evt_data }
    }
}

#[tauri::command]
pub async fn notify<T>(evt: RustEvent<T>) {
    if let Some(handle) = GLOBAL_APP_HANDLE.get() {
    }
}
