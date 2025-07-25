pub enum KMControlMode {
    Master,
    Slave,
}
pub struct KeyboardMouse {
    mode: KMControlMode,
    addr: String,
    delta: i32,
}

pub trait KeyboardMouseCtrl {
    fn send(&self);
}
