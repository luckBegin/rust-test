use std::sync::{Arc};
use async_trait::async_trait;
use tokio::net::TcpListener;
use tokio_tungstenite::{WebSocketStream, tungstenite::Message, accept_async};
use tokio::net::TcpStream;
use futures_util::stream::SplitSink;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use crate::streaming::traits::{StreamCtrl, StreamSend};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;
pub struct StreamWsServer {
    addr: String,
    clients: Arc<Mutex<Vec<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
    socket: Option<TcpListener>,
    stop_tx: Option<oneshot::Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

impl StreamWsServer {
    pub fn new(addr: String) -> Self {
        Self {
            addr,
            clients: Arc::new(Mutex::new(Vec::new())),
            socket: None,
            stop_tx: None,
            handle: None,
        }
    }
}

#[async_trait]
impl StreamCtrl for StreamWsServer {
    async fn start(&mut self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        let clients = self.clients.clone();

        let (stop_tx, stop_rx) = oneshot::channel();
        self.stop_tx = Some(stop_tx);

        let handle = tokio::spawn(async move {
            let mut stop_rx = stop_rx;
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, addr)) => {
                                println!("New WS connection from {}", addr);
                                match accept_async(stream).await {
                                    Ok(ws_stream) => {
                                        let (ws_sink, mut ws_stream_recv) = ws_stream.split();
                                        clients.lock().await.push(ws_sink);
                                    }
                                    Err(e) => {
                                        eprintln!("WS handshake failed: {}", e);
                                    }
                                }
                            }
                            Err(e) => eprintln!("Accept error: {}", e),
                        }
                    }
                    _ = &mut stop_rx => {
                        println!("Stop signal received, closing WS server");
                        break;
                    }
                }
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
        Ok(())
    }
}

#[async_trait]
impl StreamSend for StreamWsServer {
    async fn send(&self, data: Vec<u8>) -> anyhow::Result<()> {
        let mut locked = self.clients.lock().await;
        use futures::stream::{FuturesUnordered, StreamExt};
        let mut futures = FuturesUnordered::new();

        for ws_sink in locked.iter_mut() {
            futures.push(ws_sink.send(Message::Binary(data.clone().into())));
        }

        let results = futures.collect::<Vec<_>>().await;
        for res in results {
            if let Err(e) = res {
                eprintln!("Send failed: {}", e);
            }
        }

        Ok(())
    }
}
