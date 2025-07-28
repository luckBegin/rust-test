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
use rdev::Key::Print;
use resolution::current_resolution;
use tauri::async_runtime::handle;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use crate::service::tcp::tcp_server::TcpServer;
use crate::service::tcp::tcp_client::TcpClient;
use std::sync::mpsc::{Sender, channel};

#[derive(Debug, Serialize, Deserialize)]
pub enum KMEventType {
    MouseMove,
    InitMouseMove,
    Keyboard,
    MouseEvent,
    MouseBack,
    Ready,
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KmEvent<T>
where
    T: Serialize,
{
    evt_type: KMEventType,
    evt_data: T,
}

impl<T: Default + Serialize> Default for KmEvent<T> {
    fn default() -> Self {
        Self {
            evt_type: KMEventType::None,
            evt_data: T::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MouseData {
    x: i32,
    y: i32,
    x_ratio: f32,
    y_ratio: f32,
    value: String
}

impl Default for MouseData {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            x_ratio: 0.0,
            y_ratio: 0.0,
            value: "".to_string()
        }
    }
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn start_km_capture() {
    let (evt_sender, evt_receiver) = channel::<(i32, i32, f64, f64, KmEvent<MouseData>)>();
    let tcp_server = Arc::new(TcpServer::new("0.0.0.0:12345"));
    let callback = Arc::new(|peer: String, msg: String| {
        let setting = Settings::default();
        let mut enigo = Enigo::new(&setting).unwrap();
        let km_event: KmEvent<MouseData> = serde_json::from_str(&msg).expect("错误的信息");
        let evt_data = km_event.evt_data;
        let (width, height) = current_resolution().unwrap();
        match km_event.evt_type {
            KMEventType::MouseBack => {
                let y = (&evt_data.y_ratio * height as f32).round() as i32;
                enigo.move_mouse(5, y, Coordinate::Abs);
                *CURSOR_HIDE.lock().unwrap() = false;
                *IS_FIRST.lock().unwrap() = true;
                show_cursor();
            }
            _ => (),
        }
    });


    tokio::spawn({
        let tcp_server = Arc::clone(&tcp_server);
        async move {
            tcp_server.run_with_callback(callback).await;
        }
    });

    tokio::spawn({
        let tcp_server = Arc::clone(&tcp_server);
        async move {
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
                            if should_send_evt(cx, cy) {
                                &evt_sender.send((dx as i32, dy as i32, cx, cy, KmEvent::default()));
                            }
                            CallbackResult::Keep
                        }
                        CGEventType::ScrollWheel
                        | CGEventType::LeftMouseDown
                        | CGEventType::LeftMouseUp
                        | CGEventType::RightMouseDown
                        | CGEventType::RightMouseUp => mouse_action(&_type, &evt_sender),
                        _ => CallbackResult::Keep,
                    }
                },
                || {
                    CFRunLoop::run_current()
                },
            ).expect("Failed to install event tap");
        }
    });

    let handle = tokio::spawn(async move {
        for (dx, dy, cx, cy, km_evt) in evt_receiver {
            match km_evt.evt_type {
                KMEventType::None => mouse_move_handle(dx, dy, cx, cy, &tcp_server).await,
                _ => mouse_action_handle(&km_evt, &tcp_server).await
            }
        }
    });
}

pub static CURSOR_HIDE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
pub static IS_FIRST: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));
fn should_send_evt(cx: f64, cy: f64) -> bool {
    let mut should_send_evt: bool = false;
    let (width, height) = get_monitor_size();
    let delta = 3f64;
    if (!*CURSOR_HIDE.lock().unwrap()) {
        if (cx < delta) {
            hide_cursor();
            *CURSOR_HIDE.lock().unwrap() = true;
            should_send_evt = true;
        } else {
            if (*CURSOR_HIDE.lock().unwrap()) {
                show_cursor();
                *CURSOR_HIDE.lock().unwrap() = true;
            }
        }
    } else {
        should_send_evt = true
    }

    should_send_evt
}

async fn mouse_move_handle(dx: i32, dy: i32, cx: f64, cy: f64, socket: &TcpServer) {
    let (width, height) = get_monitor_size();
    let mut evt = KmEvent {
        evt_type: KMEventType::MouseMove,
        evt_data: MouseData {
            x: dx as i32,
            y: dy as i32,
            x_ratio: cx as f32 / width as f32,
            y_ratio: cy as f32 / height as f32,
            value: "".to_string()
        },
    };
    if (*IS_FIRST.lock().unwrap()) {
        evt.evt_type = KMEventType::InitMouseMove;
        *IS_FIRST.lock().unwrap() = false;
    }

    if let Ok(json) = serde_json::to_string(&evt) {
        if let Err(e) = socket.broadcast(json.as_bytes()).await {
            eprintln!("广播失败: {:?}", e);
        }
    };
}

async fn mouse_action_handle(km_evt: &KmEvent<MouseData>, socket: &TcpServer) {
    if let Ok(json) = serde_json::to_string(km_evt) {
        socket.broadcast(json.as_bytes()).await.unwrap()
    }
}

#[cfg(target_os = "macos")]
fn mouse_action(_type: &CGEventType, sender: &Sender<(i32, i32, f64, f64, KmEvent<MouseData>)>) -> CallbackResult {
    println!("type {:?}", _type);
    if !*CURSOR_HIDE.lock().unwrap() {
        return CallbackResult::Keep;
    };

    let mouse_data = match _type {
        CGEventType::LeftMouseDown => "LeftMouseDown",
        CGEventType::LeftMouseUp => "LeftMouseUp",
        CGEventType::RightMouseUp => "RightMoseUp",
        CGEventType::RightMouseDown => "RightMouseDown",
        _ => "",
    };

    let mut evt_data = MouseData::default() ;
    evt_data.value = mouse_data.to_string();
    let km_evt = KmEvent {
        evt_type: KMEventType::MouseEvent,
        evt_data
    };
    sender.send((0, 0, 0f64, 0f64, km_evt)).unwrap();
    CallbackResult::Keep
}

#[cfg(target_os = "windows")]
fn mouse_action() {}
#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn start_km_capture() {
    // let socket = TcpServer::new("127.0.0.1:12345");
    // socket.run().await;
    println!("KM capture not supported on Windows.");
}

#[tauri::command]
pub async fn start_km_udp_server() {
    let mut socket = TcpClient::connect("192.168.0.200:12345".to_string())
        .await
        .expect("连接失败");
    let mut buf = [0u8; 1024];
    let setting = Settings::default();
    let mut enigo = Enigo::new(&setting).unwrap();
    let (width, height) = current_resolution().unwrap();

    let km = KmEvent {
        evt_type: KMEventType::Ready,
        evt_data: MouseData::default(),
    };
    let km_str = serde_json::to_string(&km).unwrap();
    socket.send(km_str.as_bytes()).await;
    loop {
        match socket.receive(&mut buf).await {
            Ok((size, src_addr)) => {
                let msg = String::from_utf8_lossy(&buf[..size]);
                let evt_data: Result<KmEvent<MouseData>, _> = serde_json::from_str(&msg);
                match evt_data {
                    Ok(evt) => {
                        let data = evt.evt_data;
                        match evt.evt_type {
                            KMEventType::InitMouseMove => {
                                let y = (data.y_ratio * height as f32).round() as i32;
                                enigo.move_mouse(width - 5, y, Coordinate::Abs);
                            }
                            KMEventType::MouseMove => {
                                enigo.move_mouse(data.x, data.y, Coordinate::Rel);
                                handle_slave_mouse(&mut socket).await;
                            }
                            KMEventType::MouseEvent => {
                                println!("data type {:?}", data )
                            }
                            _ => {
                                println!("其他类型事件");
                            }
                        }
                    }
                    Err(err) => {
                        println!("解析错误: {:?}", err);
                    }
                }
            }
            Err(e) => {
                eprintln!("接收失败: {:?}", e);
                break;
            }
        }
    }
}

async fn handle_slave_mouse(tcp_client: &mut TcpClient) {
    if let OtherMouse::Position { x, y } = OtherMouse::get_mouse_position() {
        let (width, height) = current_resolution().unwrap();
        if (x >= width - 3) {
            let mut evt = KmEvent {
                evt_type: KMEventType::MouseBack,
                evt_data: MouseData {
                    x,
                    y,
                    x_ratio: x as f32 / width as f32,
                    y_ratio: y as f32 / height as f32,
                    value: "".to_string()
                },
            };
            tcp_client.send(serde_json::to_string(&evt).unwrap().as_bytes()).await.unwrap()
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
