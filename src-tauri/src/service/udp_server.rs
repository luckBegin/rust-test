use tokio::net::UdpSocket;
use tokio::sync::{broadcast, oneshot};
use tokio::task::JoinHandle;
use crate::GLOBAL::LIVE_ADDR_UDP;

pub async fn start_udp_server(
    tx_to_ws: broadcast::Sender<Vec<u8>>,
) -> (oneshot::Sender<()>, JoinHandle<()>) {
    let socket = UdpSocket::bind(LIVE_ADDR_UDP).await.unwrap();
    let (stop_tx, mut stop_rx) = oneshot::channel();
    let handle = tokio::spawn(async move {
        let mut buf = vec![0u8; 4096];
        loop {
            tokio::select! {
                res = socket.recv_from(&mut buf) => {
                    if let Ok((n, _)) = res {
                        let _ = tx_to_ws.send(buf[..n].to_vec());
                    }
                }
                _ = &mut stop_rx => {
                    break;
                }
            }
        }
    });
    (stop_tx, handle)
}
