use rdev::{listen, Event, EventType, Key, simulate, Button};

enum MouseEvt {
    Move { x: f64, y: f64 },
    Click { button: Button },
}

pub fn km_listen() {
    std::thread::spawn(move || {
        println!("process invoke");
        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error);
        }
    });
}

fn callback(evt: Event) {
    match evt.event_type {
        EventType::MouseMove { x, y } => {
            println!("鼠标移动到 ({}, {})", x, y);
        }
        EventType::ButtonPress(button) => {
            println!("鼠标点击按下: {:?}", button);
        }
        EventType::KeyPress(key) => {
            println!("键盘按下: {:?}", key);
        }
        _ => {}
    }
}
