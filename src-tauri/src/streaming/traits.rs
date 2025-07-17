use async_trait::async_trait;


#[async_trait]
pub trait StreamSend {
    async fn send(&self, data: Vec<u8>) -> anyhow::Result<()>;
}

#[async_trait]
pub trait StreamCtrl {
    async fn start(&mut self) -> anyhow::Result<()>;
    async fn stop(&mut self) -> anyhow::Result<()>;
}
