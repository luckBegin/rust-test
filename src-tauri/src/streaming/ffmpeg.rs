use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};
use crate::GLOBAL;

#[cfg(target_os = "macos")]
pub fn start_ffmpeg_udp() -> Child {
    let mut dest = PathBuf::from(&*GLOBAL::HOME_DIR);
    dest.push(GLOBAL::APP_FOLDER);
    dest.push("ffmpeg");
    dest.push("ffmpeg");

    println!("{:?}", GLOBAL::FFMPEG_UP_ADDR);
    Command::new(dest)
        .args([
            "-f", "avfoundation",
            "-capture_cursor", "1",
            "-framerate", "30",
            "-i", "0",
            "-c:v", "mpeg1video",
            "-b:v", "1000k",
            "-f", "mpegts",
            GLOBAL::FFMPEG_UP_ADDR,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start FFmpeg")
}
