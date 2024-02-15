#![feature(let_chains)]

use std::{rc::Rc, time::Duration};

use futures_util::{lock::Mutex, FutureExt};
use http_body_util::{BodyExt, Empty};
use hyper::{body::Bytes, Request};
use winio::{
    block_on,
    http::{Connector, WinioExecutor},
    spawn,
    time::timeout,
    ui::{
        Button, Canvas, Color, DrawingFontBuilder, HAlign, Point, Size, SolidColorBrush, TextBox,
        VAlign, Window,
    },
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::TRACE)
        .init();

    block_on(async {
        let window = Window::new().unwrap();
        window.set_text("Networking example").unwrap();
        window.set_size(Size::new(800.0, 600.0)).unwrap();

        let canvas = Canvas::new(&window).unwrap();
        let button = Button::new(&window).unwrap();
        button.set_text("Go").unwrap();
        let entry = TextBox::new(&window).unwrap();
        entry.set_text("https://www.example.com/").unwrap();
        spawn({
            let window = Rc::downgrade(&window);
            let canvas = Rc::downgrade(&canvas);
            let button = Rc::downgrade(&button);
            let entry = Rc::downgrade(&entry);
            async move {
                while let Some(window) = window.upgrade()
                    && let Some(canvas) = canvas.upgrade()
                    && let Some(button) = button.upgrade()
                    && let Some(entry) = entry.upgrade()
                {
                    const BWIDTH: f64 = 50.0;
                    const EHEIGHT: f64 = 20.0;

                    let csize = window.client_size().unwrap();

                    entry.set_loc(Point::new(0.0, 0.0)).unwrap();
                    entry
                        .set_size(Size::new(csize.width - BWIDTH, EHEIGHT))
                        .unwrap();

                    button
                        .set_loc(Point::new(csize.width - BWIDTH, 0.0))
                        .unwrap();
                    button.set_size(Size::new(BWIDTH, EHEIGHT)).unwrap();

                    canvas
                        .set_size(Size::new(csize.width, csize.height - EHEIGHT))
                        .unwrap();
                    canvas.set_loc(Point::new(0.0, EHEIGHT)).unwrap();
                    canvas.redraw().unwrap();

                    futures_util::select! {
                        _ = window.wait_size().fuse() => {}
                        _ = window.wait_move().fuse() => {}
                    }
                }
            }
        })
        .detach();

        let text = Rc::new(Mutex::new(None));
        let client = hyper_util::client::legacy::Builder::new(WinioExecutor)
            .set_host(true)
            .build(Connector::new(window.clone()));

        spawn({
            let window = Rc::downgrade(&window);
            let canvas = Rc::downgrade(&canvas);
            let entry = Rc::downgrade(&entry);
            let button = Rc::downgrade(&button);
            let text = text.clone();
            async move {
                loop {
                    if let Some(canvas) = canvas.upgrade() {
                        let mut text = text.lock().await;
                        if text.is_some() {
                            *text = None;
                            canvas.redraw().unwrap();
                        }
                    } else {
                        break;
                    }

                    if let Some(window) = window.upgrade()
                        && let Some(entry) = entry.upgrade()
                    {
                        let url = entry.text().unwrap();

                        let request = Request::builder()
                            .uri(url)
                            .body(Empty::<Bytes>::new())
                            .unwrap();
                        let bytes = if let Ok(res) =
                            timeout(Duration::from_secs(8), &window, client.request(request)).await
                        {
                            let response = res.unwrap();
                            response.into_body().collect().await.unwrap().to_bytes()
                        } else {
                            "Timed out.".into()
                        };

                        *text.lock().await = Some(String::from_utf8_lossy(&bytes).into_owned());
                        if let Some(canvas) = canvas.upgrade() {
                            canvas.redraw().unwrap();
                        } else {
                            break;
                        }
                    }

                    if let Some(button) = button.upgrade() {
                        button.wait_click().await;
                    } else {
                        break;
                    }
                }
            }
        })
        .detach();

        spawn({
            let canvas = Rc::downgrade(&canvas);
            async move {
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
                        text.lock()
                            .await
                            .as_ref()
                            .map(|s| s.as_str())
                            .unwrap_or("Loading..."),
                    )
                    .unwrap();
                }
            }
        })
        .detach();
        window.wait_close().await;
    })
}
