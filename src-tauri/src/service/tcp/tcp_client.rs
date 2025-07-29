use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct TcpClient {
    server_addr: SocketAddr, // 服务端地址
    socket: UdpSocket,       // 本地socket
}

impl TcpClient {
    // 创建客户端，绑定本地随机端口，指定服务端地址
    pub async fn connect(server_addr_str: String) -> std::io::Result<Self> {
        let server_addr: SocketAddr = server_addr_str.parse().expect("Invalid server address");
        // 绑定本地随机端口 (0.0.0.0:0)
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        Ok(Self {
            server_addr,
            socket,
        })
    }

    // 发送数据给服务器
    pub async fn send(&self, data: &[u8]) -> std::io::Result<()> {
        self.socket.send_to(data, &self.server_addr).await?;
        Ok(())
    }

    // 接收数据到 buffer，返回接收的字节数和发送者地址
    pub async fn receive(&self, buffer: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        let (size, src_addr) = self.socket.recv_from(buffer).await?;
        Ok((size, src_addr))
    }
}
