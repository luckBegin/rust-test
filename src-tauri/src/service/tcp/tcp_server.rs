use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

type SharedClients = Arc<Mutex<HashMap<String, Arc<Mutex<TcpStream>>>>>;

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

            let stream = Arc::new(Mutex::new(stream));
            let clients = Arc::clone(&self.clients);
            clients.lock().await.insert(peer.clone(), stream.clone());

            tokio::spawn(Self::handle_client(peer, stream, clients));
        }
    }

    async fn handle_client(peer: String, stream: Arc<Mutex<TcpStream>>, clients: SharedClients) {
        let mut buffer = [0u8; 1024];

        loop {
            let n = {
                let mut locked_stream = stream.lock().await;
                match locked_stream.read(&mut buffer).await {
                    Ok(0) => {
                        println!("Client {} disconnected", peer);
                        clients.lock().await.remove(&peer);
                        break;
                    }
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Error with client {}: {}", peer, e);
                        clients.lock().await.remove(&peer);
                        break;
                    }
                }
            };

            let msg = String::from_utf8_lossy(&buffer[..n]);
            println!("{} says: {}", peer, msg);

            let response = format!("Server received: {}\n", msg);
            let mut locked_stream = stream.lock().await;
            let _ = locked_stream.write_all(response.as_bytes()).await;
        }
    }

    pub async fn send_to(&self, peer: &str, msg: &str) -> std::io::Result<()> {
        let clients = self.clients.lock().await;
        if let Some(client) = clients.get(peer) {
            let mut stream = client.lock().await;
            stream.write_all(msg.as_bytes()).await?;
        }
        Ok(())
    }
}
