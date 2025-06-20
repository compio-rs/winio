use std::time::Duration;

use compio::{runtime::spawn, time::timeout};
use cyper::Client;
use winio::{
    App, Button, ButtonEvent, Canvas, CanvasEvent, Child, Color, ColorTheme, Component,
    ComponentSender, DrawingFontBuilder, Edit, HAlign, Layoutable, Orient, Point, Size,
    SolidColorBrush, StackPanel, VAlign, Visible, Window, WindowEvent, init, layout, start,
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new().run::<MainModel>("https://www.example.com/");
}

struct MainModel {
    window: Child<Window>,
    canvas: Child<Canvas>,
    button: Child<Button>,
    entry: Child<Edit>,
    client: Client,
    text: FetchStatus,
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
    Noop,
    Close,
    Redraw,
    Go,
    Fetch(FetchStatus),
}

impl Component for MainModel {
    type Event = ();
    type Init<'a> = &'a str;
    type Message = MainMessage;

    fn init(url: Self::Init<'_>, sender: &winio::ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "Networking example",
                size: Size::new(800.0, 600.0),
            },
            canvas: Canvas = (&window),
            button: Button = (&window) => {
                text: "Go",
            },
            entry: Edit = (&window) => {
                text: url,
            },
        }

        let client = Client::new();

        let url = url.to_string();
        spawn(fetch(client.clone(), url, sender.clone())).detach();

        window.show();

        Self {
            window,
            canvas,
            button,
            entry,
            text: FetchStatus::Loading,
            client,
        }
    }

    async fn start(&mut self, sender: &winio::ComponentSender<Self>) {
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.canvas => {
                CanvasEvent::Redraw => MainMessage::Redraw,
            },
            self.button => {
                ButtonEvent::Click => MainMessage::Go,
            },
            self.entry => {},
        }
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
            MainMessage::Noop => false,
            MainMessage::Close => {
                sender.output(());
                false
            }
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
        let csize = self.window.client_size();

        {
            let mut header_panel = layout! {
                StackPanel::new(Orient::Horizontal),
                self.entry => { grow: true },
                self.button
            };
            let mut root_panel = layout! {
                StackPanel::new(Orient::Vertical),
                header_panel,
                self.canvas => { grow: true },
            };
            root_panel.set_size(csize);
        }

        let mut ctx = self.canvas.context();
        let is_dark = ColorTheme::current() == ColorTheme::Dark;
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
            Point::zero(),
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
                Err(e) => FetchStatus::Error(format!("{e:?}")),
            }
        } else {
            FetchStatus::Timedout
        };
    sender.post(MainMessage::Fetch(status));
}
