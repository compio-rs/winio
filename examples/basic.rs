use futures_util::FutureExt;
use winio::{
    block_on,
    canvas::Canvas,
    drawing::{Point, Size},
    msgbox::{Button, MessageBox, Response},
    spawn,
    window::Window,
};

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::DEBUG)
        .init();

    block_on(async {
        let window = Window::new().unwrap();
        window.set_text("Basic example").unwrap();
        window.set_size(Size::new(800.0, 600.0)).unwrap();

        let canvas = Canvas::new(&window).unwrap();
        canvas.set_size(Size::new(400.0, 300.0)).unwrap();
        canvas.set_loc(Point::new(200.0, 150.0)).unwrap();
        spawn({
            let window = window.clone();
            let canvas = canvas.clone();
            async move {
                loop {
                    futures_util::select! {
                        _ = window.wait_size().fuse() => {}
                        _ = window.wait_move().fuse() => {}
                    }
                    canvas.set_size(window.client_size().unwrap()).unwrap();
                    canvas.redraw().unwrap();
                }
            }
        })
        .detach();
        spawn(async move {
            loop {
                canvas.wait_redraw().await;
                let _ctx = canvas.context().unwrap();
            }
        })
        .detach();
        loop {
            window.wait_close().await;
            if MessageBox::new(Some(&window))
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
