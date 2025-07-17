use tokio::sync::{Mutex, broadcast, mpsc};
use futures_util::stream::SplitSink;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Message;

type WsSink = SplitSink<WebSocketStream<TcpStream>, Message>;
type WsClients = Arc<Mutex<Vec<WsSink>>>; // 可选用 Sender 或 Sink

struct WsServer {
    clients: WsClients,
}

impl WsServer {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(vec![])),
        }
    }

    // 接收来自 UDP 的数据，然后广播给所有连接的客户端
    pub async fn send(&self, data: Vec<u8>) {
        let mut clients = self.clients.lock().await;
        clients.retain_mut(|client| {
            match client.start_send(Message::Binary(data.clone())) {
                Ok(_) => true,
                Err(e) => {
                    eprintln!("🚨 发送失败，移除客户端: {}", e);
                    false
                }
            }
        });
    }

    // 启动 WebSocket 监听
    pub async fn start(&self, listener: TcpListener, mut stop_rx: oneshot::Receiver<()>) {
        let clients = self.clients.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok((stream, _)) = listener.accept() => {
                        let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
                        let (sink, _) = ws_stream.split();

                        clients.lock().await.push(sink);
                    }
                    _ = &mut stop_rx => {
                        break;
                    }
                }
            }
        });
    }
}
