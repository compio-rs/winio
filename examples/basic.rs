use winio::{
    block_on,
    msgbox::{Button, MessageBox, Response},
    window::Window,
};

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::TRACE)
        .init();

    block_on(async {
        let window = Window::new().await.unwrap();
        window.set_text("Basic example").unwrap();
        loop {
            window.close().await;
            if MessageBox::new(Some(&window))
                .title("Basic example")
                .message("Close window?")
                .buttons(Button::Yes | Button::No)
                .show()
                .unwrap()
                == Response::Yes
            {
                window.destory().await.unwrap();
                break;
            }
        }
    })
}
