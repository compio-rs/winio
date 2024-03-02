#![feature(let_chains)]

use std::{
    rc::{Rc, Weak},
    time::Duration,
};

use compio::{runtime::spawn, time::timeout};
use cyper::Client;
use futures_util::{lock::Mutex, FutureExt};
use winio::{
    block_on, Button, Canvas, Color, DrawingFontBuilder, Edit, HAlign, Point, Size,
    SolidColorBrush, VAlign, Window,
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
        let entry = Edit::new(&window).unwrap();
        entry.set_text("https://www.example.com/").unwrap();
        spawn(render(
            Rc::downgrade(&window),
            Rc::downgrade(&canvas),
            Rc::downgrade(&button),
            Rc::downgrade(&entry),
        ))
        .detach();

        let text = Rc::new(Mutex::new(FetchStatus::Loading));
        let client = Client::builder().use_native_tls().build();

        spawn(fetch(
            Rc::downgrade(&canvas),
            Rc::downgrade(&button),
            Rc::downgrade(&entry),
            client,
            text.clone(),
        ))
        .detach();
        spawn(redraw(Rc::downgrade(&canvas), text)).detach();
        window.wait_close().await;
    })
}

async fn render(
    window: Weak<Window>,
    canvas: Weak<Canvas>,
    button: Weak<Button>,
    entry: Weak<Edit>,
) {
    while let Some(window) = window.upgrade()
        && let Some(canvas) = canvas.upgrade()
        && let Some(button) = button.upgrade()
        && let Some(entry) = entry.upgrade()
    {
        const BSIZE: Size = Size::new(60.0, 20.0);
        const CMARGIN: f64 = 10.0;

        let csize = window.client_size().unwrap();

        entry.set_loc(Point::new(0.0, 0.0)).unwrap();
        entry
            .set_size(Size::new(csize.width - BSIZE.width, BSIZE.height))
            .unwrap();

        button
            .set_loc(Point::new(csize.width - BSIZE.width, 0.0))
            .unwrap();
        button.set_size(BSIZE).unwrap();

        canvas
            .set_size(Size::new(
                csize.width,
                csize.height - BSIZE.height - CMARGIN,
            ))
            .unwrap();
        canvas
            .set_loc(Point::new(0.0, BSIZE.height + CMARGIN))
            .unwrap();
        canvas.redraw().unwrap();

        futures_util::select! {
            _ = window.wait_size().fuse() => {}
            _ = window.wait_move().fuse() => {}
        }
    }
}

enum FetchStatus {
    Loading,
    Complete(String),
    Error(String),
    Timedout,
}

async fn fetch(
    canvas: Weak<Canvas>,
    button: Weak<Button>,
    entry: Weak<Edit>,
    client: Client,
    text: Rc<Mutex<FetchStatus>>,
) {
    loop {
        if let Some(entry) = entry.upgrade() {
            let url = entry.text().unwrap();

            let status =
                if let Ok(res) = timeout(Duration::from_secs(8), client.get(url).send()).await {
                    match res {
                        Ok(response) => FetchStatus::Complete(response.text().await.unwrap()),
                        Err(e) => FetchStatus::Error(format!("{:?}", e)),
                    }
                } else {
                    FetchStatus::Timedout
                };

            *text.lock().await = status;
            if let Some(canvas) = canvas.upgrade() {
                canvas.redraw().unwrap();
            } else {
                break;
            }
        }

        if let Some(button) = button.upgrade()
            && let Some(canvas) = canvas.upgrade()
        {
            button.wait_click().await;
            let mut text = text.lock().await;
            *text = FetchStatus::Loading;
            canvas.redraw().unwrap();
        } else {
            break;
        }
    }
}

async fn redraw(canvas: Weak<Canvas>, text: Rc<Mutex<FetchStatus>>) {
    while let Some(canvas) = canvas.upgrade() {
        let ctx = canvas.wait_redraw().await.unwrap();
        let brush = SolidColorBrush::new(Color::new(127, 127, 127, 255));
        ctx.draw_str(
            &brush,
            DrawingFontBuilder::new()
                .halign(HAlign::Left)
                .valign(VAlign::Top)
                .family("Courier New")
                .size(12.0)
                .build(),
            Point::new(0.0, 0.0),
            match &*text.lock().await {
                FetchStatus::Loading => "Loading...",
                FetchStatus::Complete(s) => s.as_str(),
                FetchStatus::Error(e) => e.as_str(),
                FetchStatus::Timedout => "Timed out.",
            },
        )
        .unwrap();
    }
}
