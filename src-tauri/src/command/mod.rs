use crate::util::km_detect::{detect, HidInfo};
use crate::GLOBAL_APP_HANDLE;
use get_if_addrs::get_if_addrs;
use serde::{Deserialize, Serialize};
use tauri::async_runtime::handle;
use tauri::window;
use tauri::Emitter;
use tauri::Manager;
pub mod capture;
pub mod km_capture;
pub mod transfer;

#[tauri::command]
pub async fn devices() -> Result<Vec<HidInfo>, ()> {
    let devices = detect().await;
    Ok(devices)
}

#[derive(Serialize, Clone)]
pub enum RustEventType {
    DeviceChange,
    Download,
}
#[derive(Serialize, Clone)]
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
pub async fn notify<T>(evt: RustEvent<T>)
where
    T: Clone + Serialize,
{
    if let Some(handle) = GLOBAL_APP_HANDLE.get() {
        handle.emit("notify", evt).unwrap();
    }
}

#[tauri::command]
pub async fn find_lan_device() {
    for iface in get_if_addrs::get_if_addrs().unwrap() {
        println!("{:#?}", iface.addr.ip());
    }
}
