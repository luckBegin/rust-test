use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct TcpClient {
    address: String,
    stream: TcpStream,
}

impl TcpClient {
    pub async fn connect(address: String) -> std::io::Result<Self> {
        let stream = TcpStream::connect(address.clone()).await?;
        Ok(Self { address, stream })
    }

    pub async fn send (&mut self, data: &[u8]) -> std::io::Result<()> {
        self.stream.write_all(data).await?;
        Ok(())
    }

    pub async fn receive(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buffer).await
    }
}
