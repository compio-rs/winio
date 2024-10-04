#![feature(let_chains)]

use std::{
    io,
    path::{Path, PathBuf},
    rc::{Rc, Weak},
};

use compio::{fs::File, io::AsyncReadAtExt, runtime::spawn};
use futures_util::{FutureExt, lock::Mutex};
use winio::{
    Button, Canvas, Color, ColorTheme, DrawingFontBuilder, FileBox, HAlign, Point, Size,
    SolidColorBrush, VAlign, Window, block_on,
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

        let is_dark = window.color_theme() == ColorTheme::Dark;

        let canvas = Canvas::new(&window).unwrap();
        let button = Button::new(&window).unwrap();
        button.set_text("Choose file...").unwrap();
        spawn(render(
            Rc::downgrade(&window),
            Rc::downgrade(&canvas),
            Rc::downgrade(&button),
        ))
        .detach();

        let text = Rc::new(Mutex::new(FetchStatus::Loading));
        spawn(fetch(
            Rc::downgrade(&window),
            Rc::downgrade(&canvas),
            Rc::downgrade(&button),
            text.clone(),
        ))
        .detach();
        spawn(redraw(is_dark, Rc::downgrade(&canvas), text)).detach();
        window.wait_close().await;
    })
}

async fn render(window: Weak<Window>, canvas: Weak<Canvas>, button: Weak<Button>) {
    while let Some(window) = window.upgrade()
        && let Some(canvas) = canvas.upgrade()
        && let Some(button) = button.upgrade()
    {
        const BHEIGHT: f64 = 20.0;

        let csize = window.client_size().unwrap();

        button.set_loc(Point::new(0.0, 0.0)).unwrap();
        button.set_size(Size::new(csize.width, BHEIGHT)).unwrap();
        let bheight = button.size().unwrap().height;

        canvas
            .set_size(Size::new(csize.width, csize.height - bheight))
            .unwrap();
        canvas.set_loc(Point::new(0.0, bheight)).unwrap();
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
}

async fn read_file(path: impl AsRef<Path>) -> io::Result<String> {
    let file = File::open(path).await?;
    let (_, buffer) = file.read_to_end_at(vec![], 0).await?;
    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

async fn fetch(
    window: Weak<Window>,
    canvas: Weak<Canvas>,
    button: Weak<Button>,
    text: Rc<Mutex<FetchStatus>>,
) {
    let mut path = PathBuf::from("Cargo.toml");
    loop {
        *text.lock().await = match read_file(&path).await {
            Ok(text) => FetchStatus::Complete(text),
            Err(e) => FetchStatus::Error(format!("{:?}", e)),
        };

        if let Some(canvas) = canvas.upgrade() {
            canvas.redraw().unwrap();
        } else {
            break;
        }

        if let Some(window) = window.upgrade()
            && let Some(button) = button.upgrade()
            && let Some(canvas) = canvas.upgrade()
        {
            button.wait_click().await;
            if let Some(p) = FileBox::new()
                .title("Open file")
                .add_filter(("All files", "*.*"))
                .open(Some(&window))
                .await
                .unwrap()
            {
                path = p;
                *text.lock().await = FetchStatus::Loading;
                canvas.redraw().unwrap();
            }
        }
    }
}

async fn redraw(is_dark: bool, canvas: Weak<Canvas>, text: Rc<Mutex<FetchStatus>>) {
    while let Some(canvas) = canvas.upgrade() {
        let ctx = canvas.wait_redraw().await.unwrap();
        let brush = SolidColorBrush::new(if is_dark {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(0, 0, 0, 255)
        });
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
            },
        )
        .unwrap();
    }
}
