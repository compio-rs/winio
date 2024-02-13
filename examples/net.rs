#![feature(let_chains)]

use std::rc::Rc;

use compio_io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use futures_util::FutureExt;
use winio::{
    block_on,
    net::TcpStream,
    spawn,
    ui::{Canvas, Color, DrawingFontBuilder, HAlign, Point, Size, SolidColorBrush, VAlign, Window},
};

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::TRACE)
        .init();

    block_on(async {
        let window = Window::new().unwrap();
        window.set_text("Networking example").unwrap();
        window.set_size(Size::new(800.0, 600.0)).unwrap();

        let canvas = Canvas::new(&window).unwrap();
        spawn({
            let window = Rc::downgrade(&window);
            let canvas = Rc::downgrade(&canvas);
            async move {
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
        })
        .detach();
        spawn({
            let canvas = Rc::downgrade(&canvas);
            let window = Rc::downgrade(&window);
            async move {
                let buffer = if let Some(window) = window.upgrade() {
                    let mut stream = TcpStream::connect("www.example.com:80", &window)
                        .await
                        .unwrap();
                    stream
                        .write_all(
                            "GET / HTTP/1.1\r\nHost:www.example.com\r\nConnection: close\r\n\r\n",
                        )
                        .await
                        .unwrap();
                    stream.flush().await.unwrap();

                    let (_, buffer) = stream.read_to_end(vec![]).await.unwrap();
                    buffer
                } else {
                    b"Failed to perform async connect".to_vec()
                };
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
        })
        .detach();
        window.wait_close().await;
    })
}
