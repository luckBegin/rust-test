use crate::command;
use crate::command::{RustEvent, RustEventType};
use crate::streaming::traits::StreamCtrl;
use crate::streaming::udp::StreamUdpServer;
use crate::streaming::StreamServer;
use crate::util;
use crate::GLOBAL;
use crate::GLOBAL::{LIVE_ADDR_UDP, LIVE_ADDR_WS};
use futures_lite::Stream;
use futures_util::StreamExt;
use lazy_static::lazy_static;
use reqwest::{Client, ClientBuilder};
use serde::Serialize;
use std::fmt::format;
use std::sync::Mutex;
use std::{
    env::{self, home_dir},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tauri::command;

#[derive(Serialize)]
pub struct ScreenCap {
    support: bool,
    has_permission: bool,
}

impl ScreenCap {
    pub fn new(support: bool, has_permission: bool) -> Self {
        ScreenCap {
            support,
            has_permission,
        }
    }
}

#[tauri::command]
pub fn scp_check_if() -> ScreenCap {
    return ScreenCap::new(false, false);
}

#[tauri::command]
pub fn scp_request_permission() -> bool {
    return true;
}

#[tauri::command]
pub async fn check_if_ffmpeg() -> bool {
    let mut dest = PathBuf::from(&*GLOBAL::HOME_DIR);
    dest.push(GLOBAL::APP_FOLDER);
    let mut file_path = dest.clone();
    file_path.push("ffmpeg");

    if (dest.exists() && file_path.exists()) {
        return true;
    };

    false
}

#[tauri::command]
pub async fn download_ffmpeg() -> Result<(), String> {
    let mut dest = PathBuf::from(&*GLOBAL::HOME_DIR);
    dest.push(GLOBAL::APP_FOLDER);
    dest.push("ffmpeg");

    if !dest.exists() {
        fs::create_dir_all(&dest).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("构建客户端失败: {}", e))?;
    println!("Download Url Is {}", GLOBAL::FFMPEG_DOWNLOAD_URL);

    let mut response = client
        .get(GLOBAL::FFMPEG_DOWNLOAD_URL)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("请求失败，状态码: {}", response.status()));
    }

    let total_size = response.content_length().ok_or("无法获取文件大小")?;

    let stream = response.bytes_stream();
    tokio::pin!(stream);

    let mut fileName = dest.clone();
    fileName.push("ffmpeg.zip");
    let mut file = File::create(&fileName).map_err(|e| format!("创建文件失败: {}", e))?;

    let mut downloaded: u64 = 0;
    let mut last_percent = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("读取失败: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("写入失败: {}", e))?;

        downloaded += chunk.len() as u64;
        let percent = (downloaded as f64 / total_size as f64 * 100.0).round() as u64;
        if percent != last_percent {
            command::notify(RustEvent {
                evt_type: RustEventType::Download,
                evt_data: percent,
            })
            .await;
            last_percent = percent;
        }
    }
    println!("下载并保存成功：{}", dest.display());
    util::zip::unzip_file(&fileName, &dest);
    Ok(())
}

lazy_static! {
    static ref STREAM_SERVER: Mutex<Option<StreamServer>> = Mutex::new(None);
}

#[tauri::command]
pub async fn start_live_server() -> Result<(), String> {
    let mut server = StreamServer::new(LIVE_ADDR_UDP.to_string(), LIVE_ADDR_WS.to_string());
    server.start().await;
    let mut lock = STREAM_SERVER.lock().unwrap();

    *lock = Some(server);
    Ok(())
}

#[tauri::command]
pub async fn end_live_server() -> Result<(), String> {
    let mut lock = STREAM_SERVER.lock().unwrap();
    if let Some(mut server) = lock.take() {
        server.stop();
        Ok(())
    } else {
        Err("❌ Live server is not running".to_string())
    }
}
