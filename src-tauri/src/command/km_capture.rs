use std::net::UdpSocket;
use rdev::{listen, Event, EventType, Key};
use crate::keyboard_mouse::{km_listen};

use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapPlacement, CGEventTapOptions, CGEventType, CallbackResult,
    CGEventField, EventField,
};
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use serde::Serialize;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use crate::GLOBAL::KM_ADDR_UDP;

#[derive(Serialize)]
pub enum KMEventType {
    Mouse,
    Keyboard,
}

#[derive(Serialize)]
pub struct KmEvent<T>
where
    T: Serialize,
{
    evt_type: KMEventType,
    evt_data: T,
}

#[derive(Serialize)]
pub struct MouseData {
    x: i64,
    y: i64,
}

#[tauri::command]
pub async fn start_km_capture() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    std::thread::spawn(move || {
        CGEventTap::with_enabled(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            vec![CGEventType::MouseMoved],
            |_proxy, _type, event| {
                let dx = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_X);
                let dy = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y);
                let evt = KmEvent {
                    evt_type: KMEventType::Mouse,
                    evt_data: MouseData {
                        x: dx,
                        y: dy,
                    },
                };

                if let Ok(json) = serde_json::to_string(&evt) {
                    match socket.send_to(json.as_bytes(), "") {
                        Ok(_) => {
                            println!("x: {:?}, y: {:?}", dx, dy);
                        }
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                }

                CallbackResult::Keep
            },
            || {
                CFRunLoop::run_current()
            },
        ).expect("Failed to install event tap");
    });
}


#[tauri::command]
pub fn start_km_udp_server() {
    std::thread::spawn(|| {
        let socket = UdpSocket::bind(KM_ADDR_UDP).expect("无法绑定 UDP 端口");
        let mut buf = [0u8; 1024];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((size, src)) => {
                    let msg = String::from_utf8_lossy(&buf[..size]);
                    println!("收到来自 {} 的消息: {}", src, msg);
                }
                Err(e) => {
                    eprintln!("UDP 接收失败: {:?}", e);
                }
            }
        }
    });
}
pub fn receive_km_event() {}
