use async_hid::{HidBackend, HidResult};
use futures_lite::stream::StreamExt;
use serde::Serialize;

#[derive(Debug, Serialize)]
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
    let backend = HidBackend::default() ;

}
