use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

type SharedClients = Arc<Mutex<HashMap<String, Arc<Mutex<WriteHalf<TcpStream>>>>>>;

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

    pub async fn run(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.address).await?;
        println!("Server listening on {}", self.address);

        loop {
            let (stream, addr) = listener.accept().await?;
            let peer = addr.to_string();
            println!("New client connected: {}", peer);

            let (reader, writer) = tokio::io::split(stream);
            let writer = Arc::new(Mutex::new(writer));

            self.clients.lock().await.insert(peer.clone(), writer.clone());

            let clients_clone = Arc::clone(&self.clients);
            tokio::spawn(async move {
                Self::handle_client(peer, reader, clients_clone).await;
            });
        }
    }

    async fn handle_client(peer: String, mut reader: ReadHalf<TcpStream>, clients: SharedClients) {
        let mut buffer = [0u8; 1024];

        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => {
                    println!("Client {} disconnected", peer);
                    clients.lock().await.remove(&peer);
                    break;
                }
                Ok(n) => {
                    let msg = String::from_utf8_lossy(&buffer[..n]);
                    println!("{} says: {}", peer, msg);
                    // 如果想回复，可以调用 send_to 或 broadcast
                    // 例如简单回显:
                    // let _ = clients.lock().await.get(&peer).map(|writer| {
                    //     let mut w = writer.lock().await;
                    //     let _ = w.write_all(msg.as_bytes()).await;
                    // });
                }
                Err(e) => {
                    eprintln!("Error with client {}: {}", peer, e);
                    clients.lock().await.remove(&peer);
                    break;
                }
            }
        }
    }

    pub async fn send_to(&self, peer: &str, msg: &str) -> std::io::Result<()> {
        let clients = self.clients.lock().await;
        if let Some(writer) = clients.get(peer) {
            let mut writer = writer.lock().await;
            writer.write_all(msg.as_bytes()).await?;
        }
        Ok(())
    }

    pub async fn broadcast(&self, msg: &[u8]) -> std::io::Result<()> {
        let clients = self.clients.lock().await;
        println!("Broadcasting to {} clients", clients.len());

        for (peer, writer) in clients.iter() {
            let mut writer = writer.lock().await;
            if let Err(e) = writer.write_all(msg).await {
                eprintln!("Failed to send to {}: {:?}", peer, e);
            } else {
                println!("Sent to {}", peer);
            }
        }
        Ok(())
    }
}
