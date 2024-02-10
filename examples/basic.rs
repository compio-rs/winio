use std::rc::Rc;

use futures_util::FutureExt;
use winio::{
    block_on,
    drawing::Size,
    msgbox::{Button, MessageBox, Response},
    spawn,
    window::Window,
};

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::DEBUG)
        .init();

    block_on(async {
        let window = Rc::new(Window::new().unwrap());
        window.set_text("Basic example").unwrap();
        window.set_size(Size::new(800.0, 600.0)).unwrap();
        spawn({
            let window = window.clone();
            async move {
                loop {
                    futures_util::select! {
                        _ = window.wait_size().fuse() => {}
                        _ = window.wait_move().fuse() => {}
                    }
                    println!("Should redraw.");
                }
            }
        })
        .detach();
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
    })
}
