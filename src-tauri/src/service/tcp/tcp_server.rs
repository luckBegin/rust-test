use tokio::net::UdpSocket;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

type SharedClients = Arc<Mutex<HashMap<String, String>>>; // peer_id -> socket_addr (string)
type OnMessage = Arc<dyn Fn(String, String) + Send + Sync>;

pub struct TcpServer {
    address: String,
    clients: SharedClients,
}

impl TcpServer {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run_with_callback(&self, on_message: OnMessage) -> std::io::Result<()> {
        let socket = Arc::new(UdpSocket::bind(&self.address).await?);
        println!("UDP Server listening on {}", self.address);
        let socket_clone = socket.clone();
        let clients = self.clients.clone();
        let on_message_cb = on_message.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                match socket_clone.recv_from(&mut buf).await {
                    Ok((size, addr)) => {
                        let msg = String::from_utf8_lossy(&buf[..size]).to_string();
                        let peer = addr.to_string();
                        clients.lock().await.insert(peer.clone(), addr.to_string());
                        (on_message_cb)(peer, msg);
                    }
                    Err(e) => {
                        eprintln!("recv_from error: {:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn send_to(&self, peer: &str, msg: &str) -> std::io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?; // 临时 socket 发消息
        let clients = self.clients.lock().await;
        if let Some(addr) = clients.get(peer) {
            socket.send_to(msg.as_bytes(), addr).await?;
        }
        Ok(())
    }

    pub async fn broadcast(&self, msg: &[u8]) -> std::io::Result<()> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let clients = self.clients.lock().await;
        for (_peer, addr) in clients.iter() {
            let _ = socket.send_to(msg, addr).await;
        }

        Ok(())
    }
}
