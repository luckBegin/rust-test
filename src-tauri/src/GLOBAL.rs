use once_cell::sync::Lazy;
use std::env;
use std::path::PathBuf;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub static HOME_DIR: Lazy<String> = Lazy::new(|| {
    let path = env::home_dir().unwrap_or_else(|| PathBuf::from(""));
    path.display().to_string()
});

pub static APP_FOLDER: &str = "Keychron_Screen";
pub static LIVE_ADDR_UDP: &str = "0.0.0.0:30002";
pub static LIVE_ADDR_WS: &str = "0.0.0.0:30003";
pub static FFMPEG_UP_ADDR: &str = "udp://127.0.0.1:30002";

#[cfg(target_os = "macos")]
pub const FFMPEG_DOWNLOAD_URL: &str = "http://192.168.0.28:4001/api/ffmpeg-mac.zip";
// pub const FFMPEG_DOWNLOAD_URL: &str = "https://launcher.keychron.cn/api/ffmpeg-mac.zip";

#[cfg(target_os = "windows")]
pub const FFMPEG_DOWNLOAD_URL: &str = "https://launcher.keychron.cn/api/ffmpeg-win.zip";
