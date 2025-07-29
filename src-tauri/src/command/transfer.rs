use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use crate::{command, GLOBAL};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use crate::command::{RustEvent, RustEventType};

#[tauri::command]
pub async fn transfer_file(file_path: String) {
    tokio::spawn(async move {
        let path = Path::new(&file_path);
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let file_name_bytes = file_name.as_bytes();
        let file_name_len = file_name_bytes.len() as u32;

        let mut file = File::open(&file_path).await.unwrap();
        let total_size = file.metadata().await.unwrap().len();

        let mut stream = TcpStream::connect("192.178.0.200:30006").await.unwrap();

        stream.write_all(&file_name_len.to_be_bytes()).await.unwrap();

        stream.write_all(file_name_bytes).await.unwrap();

        let mut sent: u64 = 0;
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer).await.unwrap();
            if bytes_read == 0 {
                break;
            }

            stream.write_all(&buffer[..bytes_read]).await.unwrap();
            sent += bytes_read as u64;
            let percent = (sent as f64 / total_size as f64) * 100.0;
            command::notify(RustEvent {
                evt_type: RustEventType::Download,
                evt_data: percent,
            }).await;
        }

        println!("发送完成: 文件名={} 总大小={}字节", file_name, total_size);
    });
}


#[tauri::command]
pub async fn receive_file() {
    use tokio::io::AsyncReadExt;

    tokio::spawn(async move {
        let listener = TcpListener::bind("0.0.0.0:30006").await.unwrap();
        println!("TCP 监听中...");

        loop {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    println!("收到连接: {:?}", addr);

                    let mut len_buf = [0u8; 4];
                    if let Err(e) = stream.read_exact(&mut len_buf).await {
                        eprintln!("读取文件名长度失败: {:?}", e);
                        continue;
                    }
                    let file_name_len = u32::from_be_bytes(len_buf) as usize;

                    let mut name_buf = vec![0u8; file_name_len];
                    if let Err(e) = stream.read_exact(&mut name_buf).await {
                        eprintln!("读取文件名失败: {:?}", e);
                        continue;
                    }
                    let file_name = match String::from_utf8(name_buf) {
                        Ok(name) => name,
                        Err(e) => {
                            eprintln!("文件名解析失败: {:?}", e);
                            continue;
                        }
                    };

                    let mut file_buf = Vec::new();
                    if let Err(e) = stream.read_to_end(&mut file_buf).await {
                        eprintln!("读取文件内容失败: {:?}", e);
                        continue;
                    }


                    let mut full_path = PathBuf::from(&*GLOBAL::HOME_DIR);
                    full_path.push(GLOBAL::APP_FOLDER);
                    full_path.push(file_name.clone());


                    if let Err(e) = tokio::fs::write(&full_path, &file_buf).await {
                        eprintln!("写文件失败: {:?}", e);
                        continue;
                    }

                    println!("已保存文件: {:?}", full_path);
                }

                Err(e) => eprintln!("接收连接失败: {:?}", e),
            }
        }
    });
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn open_folder() {
    let mut full_path = PathBuf::from(&*GLOBAL::HOME_DIR);
    full_path.push(GLOBAL::APP_FOLDER);
    std::process::Command::new("open")
        .arg(full_path)
        .spawn()
        .expect("打开文件夹失败");
}
#[cfg(target_os = "windows")]
#[tauri::command]
pub fn open_folder() {
    let mut full_path = PathBuf::from(&*GLOBAL::HOME_DIR);
    full_path.push(GLOBAL::APP_FOLDER);
    std::process::Command::new("explorer")
        .arg(full_path)
        .spawn()
        .expect("打开文件夹失败");
}
