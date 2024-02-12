use futures_util::FutureExt;
use winio::{
    block_on,
    canvas::{BrushPen, Canvas, SolidColorBrush},
    drawing::{Color, DrawingFontBuilder, HAlign, Point, Rect, Size, VAlign},
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
        spawn({
            let window = window.clone();
            let canvas = canvas.clone();
            async move {
                loop {
                    let csize = window.client_size().unwrap();
                    canvas.set_size(csize / 2.0).unwrap();
                    canvas
                        .set_loc(Point::new(csize.width / 4.0, csize.height / 4.0))
                        .unwrap();
                    canvas.redraw().unwrap();
                    futures_util::select! {
                        _ = window.wait_size().fuse() => {}
                        _ = window.wait_move().fuse() => {}
                    }
                }
            }
        })
        .detach();
        spawn(async move {
            loop {
                canvas.wait_redraw().await;
                let ctx = canvas.context().unwrap();
                let size = canvas.size().unwrap();
                let brush = SolidColorBrush::new(Color::new(0, 0, 0, 255));
                ctx.draw_ellipse(
                    &BrushPen::new(brush.clone(), 1.0),
                    Rect::new(Point::new(size.width / 4.0, size.height / 4.0), size / 2.0),
                )
                .unwrap();
                ctx.draw_str(
                    &brush,
                    DrawingFontBuilder::new()
                        .halign(HAlign::Center)
                        .valign(VAlign::Center)
                        .family("Segoe UI")
                        .size(12.0)
                        .build(),
                    Point::new(size.width / 2.0, size.height / 2.0),
                    "Hello world!",
                )
                .unwrap();
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
