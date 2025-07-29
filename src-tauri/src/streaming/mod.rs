use crate::streaming::ffmpeg::start_ffmpeg_udp;
use crate::streaming::traits::{StreamCtrl, StreamSend};
use crate::streaming::udp::StreamUdpServer;
use crate::streaming::ws::StreamWsServer;
use std::cmp::PartialEq;
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, Mutex};

pub mod ffmpeg;
pub mod traits;
pub mod udp;
pub mod ws;
#[derive(Clone, Debug, PartialEq)]
pub enum StreamEvtType {
    UDP,
    WS,
    FFMPEG,
}

#[derive(Clone, Debug)]
pub struct StreamEvt {
    evt_type: StreamEvtType,
    evt_data: Vec<u8>,
}

pub struct StreamServer {
    udp: StreamUdpServer,
    ws: Arc<Mutex<StreamWsServer>>,
    tx: broadcast::Sender<StreamEvt>,
    rx: broadcast::Receiver<StreamEvt>,
    ffmpeg: Option<Child>,
}

impl StreamServer {
    pub fn new(udp_addr: String, ws_addr: String) -> Self {
        let (tx, rx) = broadcast::channel(256);
        Self {
            tx: tx.clone(),
            rx,
            udp: StreamUdpServer::new(udp_addr, tx.clone()),
            ws: Arc::new(Mutex::from(StreamWsServer::new(ws_addr))),
            ffmpeg: None,
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.udp.start().await.expect("Boot Fail");
        {
            let mut ws_locked = self.ws.lock().await;
            ws_locked.start().await.expect("Boot Fail");
        }

        let child = start_ffmpeg_udp();
        self.ffmpeg = Some(child);

        let mut rx = self.rx.resubscribe();
        let ws = Arc::clone(&self.ws);

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(evt) => {
                        if evt.evt_type == StreamEvtType::UDP {
                            let mut ws_locked = ws.lock().await;
                            if let Err(e) = ws_locked.send(evt.evt_data.clone()).await {
                                eprintln!("❌ WS 发送失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        match e {
                            RecvError::Closed => {
                                eprintln!("发送端已经关闭，无法再接收数据");
                                break; // 可以退出循环
                            }
                            RecvError::Lagged(n) => {
                                eprintln!("消息丢失了 {} 条", n);
                                // 这里可以选择继续循环接收或者做其他处理
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&mut self) {
        &self.udp.stop().await.expect("Stop Fail");
        let ws = Arc::clone(&self.ws);
        let mut ws_guard = ws.lock().await;
        ws_guard.stop().await.expect("Stop Fail");
    }
}
