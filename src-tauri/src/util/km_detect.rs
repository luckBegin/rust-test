use std::collections::HashMap;
use async_hid::{Device, DeviceEvent, DeviceId, HidBackend, HidResult};
use futures_lite::stream::StreamExt;
use serde::Serialize;
use log::log;

use crate::{command};
use crate::command::{RustEvent, RustEventType};
use crate::GLOBAL_APP_HANDLE;

#[derive(Debug, Serialize, Clone)]
pub struct HidInfo {
    pub vid: u16,
    pub pid: u16,
    pub name: String,
    pub usage: u16,
    pub usage_page: u16,
}
pub async fn detect() -> Vec<HidInfo> {
    let backend = HidBackend::default();
    let devices = backend.enumerate().await;

    let mut result = Vec::new();
    match devices {
        Ok(mut stream) => {
            while let Some(device) = stream.next().await {
                if (device.vendor_id == 0x3434 || device.vendor_id == 0x362d) {
                    result.push(HidInfo {
                        vid: device.vendor_id,
                        pid: device.product_id,
                        name: device.name.clone(),
                        usage: device.usage_id,
                        usage_page: device.usage_page,
                    })
                }
            }
        }
        Err(err) => {}
    };
    result
}

pub async fn watch_device() {
    let backend = HidBackend::default();
    let Ok(mut watcher) = backend.watch() else { return };
    while let Some(event) = watcher.next().await {
        match event {
            DeviceEvent::Connected(id) => {
                match backend.query_devices(&id).await {
                    Ok(devices) => {
                        let mut map: HashMap<u32, HidInfo> = HashMap::new();
                        for device in devices {
                            let vp_id = (device.vendor_id as u32) << 16 | (device.product_id as u32);
                            map.entry(vp_id).or_insert(HidInfo {
                                pid: device.product_id,
                                vid: device.vendor_id,
                                usage_page: device.usage_page,
                                usage: device.usage_id,
                                name: device.name.clone(),
                            });
                        }
                        let list: Vec<HidInfo> = map.into_values().collect();
                        command::notify(RustEvent {
                            evt_type: RustEventType::DeviceChange,
                            evt_data: list,
                        }).await;
                    }
                    Err(error) => {
                        println!("Query Device Info Error With Device Id {:?}", id);
                    }
                }
            }
            DeviceEvent::Disconnected(id) => {}
        }
    }
}
