use std::rc::Rc;

use winio::{
    block_on,
    drawing::Size,
    msgbox::{Button, MessageBox, Response},
    window::Window,
};

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::DEBUG)
        .init();

    block_on(async {
        let window = Rc::new(Window::new().await.unwrap());
        let task = {
            let window = window.clone();
            async move {
                loop {
                    window.wait_close().await;
                    if MessageBox::new(Some(window.as_ref()))
                        .title("Basic example")
                        .message("Close window?")
                        .buttons(Button::Yes | Button::No)
                        .show()
                        .unwrap()
                        == Response::Yes
                    {
                        break;
                    }
                }
            }
        };
        window.set_text("Basic example").unwrap();
        window.set_size(Size::new(800.0, 600.0)).unwrap();
        task.await;
    })
}
