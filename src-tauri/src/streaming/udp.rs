use crate::streaming::traits::StreamCtrl;
use crate::streaming::{StreamEvt, StreamEvtType};
use async_trait::async_trait;
use std::clone::Clone;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, oneshot};
pub struct StreamUdpServer {
    socket: Option<Arc<UdpSocket>>,
    addr: String,
    tx: broadcast::Sender<StreamEvt>,
    stop_tx: Option<oneshot::Sender<()>>,
}

impl StreamUdpServer {
    pub fn new(addr: String, tx: broadcast::Sender<StreamEvt>) -> Self {
        Self {
            addr,
            socket: None,
            tx,
            stop_tx: None,
        }
    }
}

#[async_trait]
impl StreamCtrl for StreamUdpServer {
    async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        self.socket = None;
        Ok(())
    }

    async fn start(&mut self) -> anyhow::Result<()> {
        let socket = UdpSocket::bind(&self.addr).await?;
        let socket = Arc::new(socket);
        self.socket = Some(Arc::clone(&socket));

        let socket_clone = Arc::clone(&socket);
        let tx = self.tx.clone();

        let (stop_tx, mut stop_rx) = oneshot::channel();
        self.stop_tx = Some(stop_tx);

        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            loop {
                tokio::select! {
                    _ = &mut stop_rx => {
                        println!("🛑 UDP Server 收到退出信号");
                        break;
                    }
                    result = socket_clone.recv_from(&mut buf) => {
                        match result {
                            Ok((n, _)) => {
                                let _ = tx.send(StreamEvt {
                                    evt_type: StreamEvtType::UDP,
                                    evt_data: buf[..n].to_vec(),
                                });
                            }
                            Err(e) => {
                                eprintln!("❌ 接收失败: {}", e);
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }
}
