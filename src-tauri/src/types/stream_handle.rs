use std::process::Child;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub struct StreamHandle {
    pub udp_stop: Option<oneshot::Sender<()>>,
    pub ffmpeg_child: Option<Child>,
    pub ws_stop: Option<oneshot::Receiver<()>>,
    pub relay_handle: Option<JoinHandle<()>>,
}
