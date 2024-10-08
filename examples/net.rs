use std::time::Duration;

use compio::{runtime::spawn, time::timeout};
use cyper::Client;
use winio::{
    App, Button, ButtonEvent, Canvas, CanvasEvent, CanvasMessage, Child, Color, ColorTheme,
    Component, ComponentSender, DrawingFontBuilder, Edit, HAlign, Point, Size, SolidColorBrush,
    VAlign, Window, WindowEvent,
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new().run::<MainModel>("https://www.example.com/", &());
}

struct MainModel {
    window: Child<Window>,
    canvas: Child<Canvas>,
    button: Child<Button>,
    entry: Child<Edit>,
    client: Client,
    text: FetchStatus,
    is_dark: bool,
}

#[derive(Debug)]
enum FetchStatus {
    Loading,
    Complete(String),
    Error(String),
    Timedout,
}

#[derive(Debug)]
enum MainMessage {
    Close,
    QueueRedraw,
    Redraw,
    Go,
    Fetch(FetchStatus),
}

impl Component for MainModel {
    type Event = ();
    type Init = &'static str;
    type Message = MainMessage;
    type Root = ();

    fn init(
        counter: Self::Init,
        _root: &Self::Root,
        sender: &winio::ComponentSender<Self>,
    ) -> Self {
        let mut window = Child::<Window>::init((), &());
        window.set_text("Networking example");
        window.set_size(Size::new(800.0, 600.0));

        let is_dark = ColorTheme::current() == ColorTheme::Dark;

        let canvas = Child::<Canvas>::init((), &window);
        let mut button = Child::<Button>::init((), &window);
        button.set_text("Go");
        let mut entry = Child::<Edit>::init((), &window);
        entry.set_text(counter);

        let client = Client::new();

        spawn(fetch(client.clone(), counter.to_string(), sender.clone())).detach();

        Self {
            window,
            canvas,
            button,
            entry,
            text: FetchStatus::Loading,
            client,
            is_dark,
        }
    }

    async fn start(&mut self, sender: &winio::ComponentSender<Self>) {
        let fut_window = self.window.start(sender, |e| match e {
            WindowEvent::Close => Some(MainMessage::Close),
            WindowEvent::Move | WindowEvent::Resize => Some(MainMessage::QueueRedraw),
            _ => None,
        });
        let fut_canvas = self.canvas.start(sender, |e| match e {
            CanvasEvent::Redraw => Some(MainMessage::Redraw),
            _ => None,
        });
        let fut_button = self.button.start(sender, |e| match e {
            ButtonEvent::Click => Some(MainMessage::Go),
            _ => None,
        });
        let fut_entry = self.entry.start(sender, |_| None);
        futures_util::future::join4(fut_window, fut_canvas, fut_button, fut_entry).await;
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &winio::ComponentSender<Self>,
    ) -> bool {
        futures_util::future::join4(
            self.window.update(),
            self.canvas.update(),
            self.button.update(),
            self.entry.update(),
        )
        .await;
        match message {
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::QueueRedraw => self.canvas.emit(CanvasMessage::Redraw).await,
            MainMessage::Redraw => true,
            MainMessage::Go => {
                spawn(fetch(
                    self.client.clone(),
                    self.entry.text(),
                    sender.clone(),
                ))
                .detach();
                false
            }
            MainMessage::Fetch(status) => {
                self.text = status;
                true
            }
        }
    }

    fn render(&mut self, _sender: &winio::ComponentSender<Self>) {
        const BSIZE: Size = Size::new(60.0, 20.0);

        let csize = self.window.client_size();

        self.entry.set_loc(Point::new(0.0, 0.0));
        self.entry
            .set_size(Size::new(csize.width - BSIZE.width, BSIZE.height));
        let bheight = self.entry.size().height;

        self.button
            .set_loc(Point::new(csize.width - BSIZE.width, 0.0));
        self.button.set_size(Size::new(BSIZE.width, bheight));

        self.canvas
            .set_size(Size::new(csize.width, csize.height - bheight));
        self.canvas.set_loc(Point::new(0.0, bheight));

        let ctx = self.canvas.context();
        let brush = SolidColorBrush::new(if self.is_dark {
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
            match &self.text {
                FetchStatus::Loading => "Loading...",
                FetchStatus::Complete(s) => s.as_str(),
                FetchStatus::Error(e) => e.as_str(),
                FetchStatus::Timedout => "Timed out.",
            },
        );
    }
}

async fn fetch(client: Client, url: String, sender: ComponentSender<MainModel>) {
    sender.post(MainMessage::Fetch(FetchStatus::Loading));
    let status =
        if let Ok(res) = timeout(Duration::from_secs(8), client.get(url).unwrap().send()).await {
            match res {
                Ok(response) => FetchStatus::Complete(response.text().await.unwrap()),
                Err(e) => FetchStatus::Error(format!("{:?}", e)),
            }
        } else {
            FetchStatus::Timedout
        };
    sender.post(MainMessage::Fetch(status));
}
