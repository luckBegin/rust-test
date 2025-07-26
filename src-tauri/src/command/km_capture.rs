use std::error::Error;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use rdev::{listen, Event, EventType, Key};
use enigo::*;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use crate::GLOBAL::KM_ADDR_UDP;
use once_cell::sync::Lazy;
use mouse_position::mouse_position::Mouse as OtherMouse;
use std::io::ErrorKind;

#[cfg(target_os = "macos")]
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapPlacement, CGEventTapOptions, CGEventType, CallbackResult,
    CGEventField, EventField,
};

#[cfg(target_os = "macos")]
use core_graphics::display::{CGMainDisplayID, CGDisplay};

#[cfg(target_os = "macos")]
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
#[cfg(target_os = "macos")]
use core_graphics::display::CGPoint;
#[cfg(target_os = "macos")]
use objc::runtime::protocol_conformsToProtocol;
use resolution::current_resolution;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use crate::service::tcp::tcp_server::TcpServer;

#[derive(Debug, Serialize, Deserialize)]
pub enum KMEventType {
    MouseMove,
    InitMouseMove,
    Keyboard,
    MouseClickLeft,
    MouseClickRight,
    MouseBack
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KmEvent<T>
where
    T: Serialize,
{
    evt_type: KMEventType,
    evt_data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MouseData {
    x: i32,
    y: i32,
    x_ratio: f32,
    y_ratio: f32,
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn start_km_capture() {
    let socket = Arc::new(Mutex::new(UdpSocket::bind(KM_ADDR_UDP).unwrap()));
    socket.lock().unwrap().set_nonblocking(true).unwrap();
    let socket_1 = Arc::clone(&socket);
    std::thread::spawn(move || {
        CGEventTap::with_enabled(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default,
            vec![
                CGEventType::MouseMoved,
                CGEventType::LeftMouseDown,
                CGEventType::LeftMouseUp,
                CGEventType::RightMouseDown,
                CGEventType::RightMouseUp,
                CGEventType::ScrollWheel,
            ],
            move |_proxy, _type, event| {
                match _type {
                    CGEventType::MouseMoved => {
                        let dx = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_X);
                        let dy = event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y);
                        let CGPoint { x: cx, y: cy } = event.location();
                        if (should_send_evt(cx, cy)) {
                            return mouse_move_handle(dx as i32, dy as i32, cx, cy, &socket_1.lock().unwrap());
                        };
                        CallbackResult::Keep
                    }
                    CGEventType::ScrollWheel
                    | CGEventType::LeftMouseDown
                    | CGEventType::LeftMouseUp
                    | CGEventType::RightMouseDown
                    | CGEventType::RightMouseUp => mouse_action(),
                    _ => CallbackResult::Keep,
                }
            },
            || {
                CFRunLoop::run_current()
            },
        ).expect("Failed to install event tap");
    });
}

pub static MOUSE_POS: Lazy<Mutex<i32>> = Lazy::new(|| Mutex::new(0));
pub static CURSOR_HIDE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));

fn should_send_evt(cx: f64, cy: f64) -> bool {
    let mut should_send_evt: bool = false;
    let (width, height) = get_monitor_size();
    let delta = 3f64;
    if (*MOUSE_POS.lock().unwrap() == 0) {
        if (cx < delta) {
            hide_cursor();
            *CURSOR_HIDE.lock().unwrap() = true;
            should_send_evt = true;
        } else {
            if (*CURSOR_HIDE.lock().unwrap()) {
                show_cursor();
                *CURSOR_HIDE.lock().unwrap() = false;
            }
        }
    } else {
        should_send_evt = true
    }

    should_send_evt
}


#[cfg(target_os = "macos")]
fn mouse_move_handle(dx: i32, dy: i32, cx: f64, cy: f64, socket: &UdpSocket) -> CallbackResult {
    let (width, height) = get_monitor_size();
    let mut evt = KmEvent {
        evt_type: KMEventType::MouseMove,
        evt_data: MouseData {
            x: dx as i32,
            y: dy as i32,
            x_ratio: cx as f32 / width as f32,
            y_ratio: cy as f32 / height as f32,
        },
    };

    if (*MOUSE_POS.lock().unwrap() == 0) {
        evt.evt_type = KMEventType::InitMouseMove;
    }
    if let Ok(json) = serde_json::to_string(&evt) {
        match socket.send_to(json.as_bytes(), "192.168.0.28:30004") {
            Ok(_) => {
                *MOUSE_POS.lock().unwrap() += dx;
                // println!("x: {:?}, y: {:?}, diff {:?}", dx, dy, *MOUSE_POS.lock().unwrap());
            }
            Err(e) => {
                println!("Send Error: {:?}", e);
            }
        }
    };
    CallbackResult::Drop
}

#[cfg(target_os = "windows")]
fn mouse_move_handle(dx: i32, dy: i32, cx: f64, cy: f64, socket: &UdpSocket) {}

#[cfg(target_os = "macos")]
fn mouse_action() -> CallbackResult {
    CallbackResult::Keep
}

#[cfg(target_os = "windows")]
fn mouse_action() {}
#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn start_km_capture() {
    let socket = TcpServer::new("127.0.0.1:12345");
    socket.run().await;
    println!("KM capture not supported on Windows.");
}

#[tauri::command]
pub fn start_km_udp_server() {
    std::thread::spawn(|| {
        let socket = UdpSocket::bind(KM_ADDR_UDP).expect("无法绑定 UDP 端口");
        let mut buf = [0u8; 1024];
        let setting = Settings::default();
        let mut enigo = Enigo::new(&setting).unwrap();
        let (width, height) = current_resolution().unwrap();
        loop {
            match socket.recv_from(&mut buf) {
                Ok((size, src)) => {
                    let msg = String::from_utf8_lossy(&buf[..size]);
                    let evt_data: Result<KmEvent<MouseData>, _> = serde_json::from_str(&msg);
                    match evt_data {
                        Ok(evt) => {
                            let data = evt.evt_data;
                            match evt.evt_type {
                                KMEventType::InitMouseMove => {
                                    let y = (&data.y_ratio * height as f32).round() as i32;
                                    println!("收到消息: Y: {:?}", y);
                                    enigo.move_mouse(width, y, Coordinate::Abs);
                                }
                                KMEventType::MouseMove => {
                                    enigo.move_mouse(data.x, data.y, Coordinate::Rel);
                                    handle_slave_mouse(&socket);
                                    println!("收到来自 {} 的消息: type: {:?}, data: {:?}", src, evt.evt_type, data);
                                }
                                _ => {
                                    println!("receive event")
                                }
                            }
                        }
                        Err(err) => {
                            println!("Parse Error, {:?}", err)
                        }
                    }
                }
                Err(e) => {
                    eprintln!("UDP 接收失败: {:?}", e);
                }
            }
        }
    });
}

fn handle_slave_mouse (udp_socket: &UdpSocket) {
    if let OtherMouse::Position {x,y} = OtherMouse::get_mouse_position() {
        let (width, height) = current_resolution().unwrap();
        if( x >= width - 3 ) {
            let mut evt = KmEvent {
                evt_type: KMEventType::MouseBack,
                evt_data: MouseData {
                    x, y,
                    x_ratio: x as f32  / width as f32,
                    y_ratio: y as f32 / height as f32,
                },
            };
            println!("Border Detect") ;
            udp_socket.send_to(&serde_json::to_string(&evt).unwrap().as_bytes(), "192.168.0.200:30004").unwrap();
        }
    }
}
pub fn receive_km_event() {}

pub fn get_monitor_size() -> (i32, i32) {
    current_resolution().expect("Resolution Failed")
}

#[cfg(target_os = "macos")]
fn hide_cursor() {
    unsafe {
        let main_display = CGDisplay { id: CGMainDisplayID() };
        main_display.hide_cursor().unwrap();
        println!("hide mouse")
    }
}

#[cfg(target_os = "macos")]
fn show_cursor() {
    unsafe {
        let main_display = CGDisplay { id: CGMainDisplayID() };
        main_display.show_cursor().unwrap();
    }
}


#[cfg(target_os = "windows")]
fn hide_cursor() {}

#[cfg(target_os = "windows")]
fn show_cursor() {}
