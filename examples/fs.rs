#![feature(let_chains)]

use std::rc::{Rc, Weak};

use compio_io::AsyncReadAtExt;
use futures_util::FutureExt;
use winio::{
    block_on,
    fs::File,
    spawn,
    ui::{Canvas, Color, DrawingFontBuilder, HAlign, Point, Size, SolidColorBrush, VAlign, Window},
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::DEBUG)
        .init();

    block_on(async {
        let window = Window::new().unwrap();
        window.set_text("File IO example").unwrap();
        window.set_size(Size::new(800.0, 600.0)).unwrap();

        let canvas = Canvas::new(&window).unwrap();
        spawn(render(Rc::downgrade(&window), Rc::downgrade(&canvas))).detach();
        spawn(redraw(Rc::downgrade(&canvas))).detach();
        window.wait_close().await;
    })
}

async fn render(window: Weak<Window>, canvas: Weak<Canvas>) {
    while let Some(window) = window.upgrade()
        && let Some(canvas) = canvas.upgrade()
    {
        let csize = window.client_size().unwrap();
        canvas.set_size(csize * 0.9).unwrap();
        canvas
            .set_loc(Point::new(csize.width * 0.05, csize.height * 0.05))
            .unwrap();
        canvas.redraw().unwrap();
        futures_util::select! {
            _ = window.wait_size().fuse() => {}
            _ = window.wait_move().fuse() => {}
        }
    }
}

async fn redraw(canvas: Weak<Canvas>) {
    let file = File::open("Cargo.toml").unwrap();
    let (_, buffer) = file.read_to_end_at(vec![], 0).await.unwrap();
    let text = std::str::from_utf8(&buffer).unwrap();

    if let Some(canvas) = canvas.upgrade() {
        canvas.redraw().unwrap();
    }

    while let Some(canvas) = canvas.upgrade() {
        canvas.wait_redraw().await;
        let ctx = canvas.context().unwrap();
        let brush = SolidColorBrush::new(Color::new(0, 0, 0, 255));
        ctx.draw_str(
            &brush,
            DrawingFontBuilder::new()
                .halign(HAlign::Left)
                .valign(VAlign::Top)
                .family("Consolas")
                .size(12.0)
                .build(),
            Point::new(0.0, 0.0),
            text,
        )
        .unwrap();
    }
}
