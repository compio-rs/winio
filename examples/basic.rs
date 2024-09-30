#![feature(let_chains)]

use std::{
    cell::Cell,
    rc::{Rc, Weak},
    time::Duration,
};

use compio::{runtime::spawn, time::interval};
use futures_util::FutureExt;
use winio::{
    BrushPen, Canvas, Color, ColorTheme, DrawingFontBuilder, HAlign, MessageBox, MessageBoxButton,
    MessageBoxResponse, MessageBoxStyle, Point, Rect, Size, SolidColorBrush, VAlign, Window,
    block_on,
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::DEBUG)
        .init();

    block_on(async {
        let window = Window::new().unwrap();
        window.set_text("Basic example").unwrap();
        window.set_size(Size::new(800.0, 600.0)).unwrap();

        let is_dark = window.color_theme() == ColorTheme::Dark;

        let canvas = Canvas::new(&window).unwrap();
        let counter = Rc::new(Cell::new(0usize));

        spawn(render(Rc::downgrade(&window), Rc::downgrade(&canvas))).detach();
        spawn(track(Rc::downgrade(&canvas))).detach();
        spawn(tick(Rc::downgrade(&canvas), counter.clone())).detach();
        spawn(redraw(is_dark, Rc::downgrade(&canvas), counter)).detach();
        wait_close(window).await;
    })
}

async fn track(canvas: Weak<Canvas>) {
    while let Some(canvas) = canvas.upgrade() {
        futures_util::select! {
            m = canvas.wait_mouse_down().fuse() => println!("{:?}", m),
            m = canvas.wait_mouse_up().fuse() => println!("{:?}", m),
            p = canvas.wait_mouse_move().fuse() => println!("{:?}", p.unwrap()),
        }
    }
}

async fn render(window: Weak<Window>, canvas: Weak<Canvas>) {
    while let Some(window) = window.upgrade()
        && let Some(canvas) = canvas.upgrade()
    {
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

async fn tick(canvas: Weak<Canvas>, counter: Rc<Cell<usize>>) {
    let mut interval = interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        counter.set(counter.get() + 1);
        if let Some(canvas) = canvas.upgrade() {
            canvas.redraw().unwrap();
        } else {
            break;
        }
    }
}

async fn redraw(is_dark: bool, canvas: Weak<Canvas>, counter: Rc<Cell<usize>>) {
    while let Some(canvas) = canvas.upgrade() {
        let ctx = canvas.wait_redraw().await.unwrap();
        let size = canvas.size().unwrap();
        let brush = SolidColorBrush::new(if is_dark {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(0, 0, 0, 255)
        });
        ctx.draw_ellipse(
            BrushPen::new(brush.clone(), 1.0),
            Rect::new(Point::new(size.width / 4.0, size.height / 4.0), size / 2.0),
        )
        .unwrap();
        ctx.draw_str(
            &brush,
            DrawingFontBuilder::new()
                .halign(HAlign::Center)
                .valign(VAlign::Center)
                .family("Arial")
                .size(12.0)
                .build(),
            Point::new(size.width / 2.0, size.height / 2.0),
            format!("Hello world!\nRunning: {}s", counter.get()),
        )
        .unwrap();
    }
}

async fn wait_close(window: Rc<Window>) {
    loop {
        window.wait_close().await;
        if MessageBox::new()
            .title("Basic example")
            .message("Close window?")
            .style(MessageBoxStyle::Info)
            .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
            .show(Some(&window))
            .await
            .unwrap()
            == MessageBoxResponse::Yes
        {
            break;
        }
    }
}
