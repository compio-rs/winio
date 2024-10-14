use std::time::Duration;

use compio::{runtime::spawn, time::timeout};
use cyper::Client;
use taffy::{NodeId, TaffyTree};
use winio::{
    App, Button, ButtonEvent, Canvas, Child, Color, ColorTheme, Component, ComponentSender,
    DrawingFontBuilder, Edit, HAlign, Layoutable, Point, Rect, Size, SolidColorBrush, VAlign,
    Window, WindowEvent,
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
            WindowEvent::Move | WindowEvent::Resize => Some(MainMessage::Redraw),
            _ => None,
        });
        let fut_button = self.button.start(sender, |e| match e {
            ButtonEvent::Click => Some(MainMessage::Go),
            _ => None,
        });
        let fut_entry = self.entry.start(sender, |_| None);
        futures_util::future::join3(fut_window, fut_button, fut_entry).await;
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

        let (erect, brect, crect) = Layout::new(self.entry.preferred_size().height).compute(csize);
        self.entry.set_rect(erect);
        self.button.set_rect(brect);
        self.canvas.set_rect(crect);

        let mut ctx = self.canvas.context();
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
                Err(e) => FetchStatus::Error(format!("{:?}", e)),
            }
        } else {
            FetchStatus::Timedout
        };
    sender.post(MainMessage::Fetch(status));
}

struct Layout {
    taffy: TaffyTree,
    canvas: NodeId,
    button: NodeId,
    entry: NodeId,
    root: NodeId,
}

impl Layout {
    pub fn new(bheight: f64) -> Self {
        let mut taffy = TaffyTree::new();
        let entry = taffy
            .new_leaf(taffy::Style {
                size: taffy::Size::auto(),
                flex_grow: 1.0,
                ..Default::default()
            })
            .unwrap();
        let button = taffy
            .new_leaf(taffy::Style {
                size: taffy::Size {
                    width: taffy::Dimension::Length(60.0),
                    height: taffy::Dimension::Auto,
                },
                ..Default::default()
            })
            .unwrap();
        let header = taffy
            .new_with_children(
                taffy::Style {
                    size: taffy::Size {
                        width: taffy::Dimension::Percent(1.0),
                        height: taffy::Dimension::Length(bheight as _),
                    },
                    flex_direction: taffy::FlexDirection::Row,
                    ..Default::default()
                },
                &[entry, button],
            )
            .unwrap();
        let canvas = taffy
            .new_leaf(taffy::Style {
                size: taffy::Size {
                    width: taffy::Dimension::Percent(1.0),
                    height: taffy::Dimension::Auto,
                },
                flex_grow: 1.0,
                ..Default::default()
            })
            .unwrap();
        let root = taffy
            .new_with_children(
                taffy::Style {
                    size: taffy::Size::from_percent(1.0, 1.0),
                    flex_direction: taffy::FlexDirection::Column,
                    ..Default::default()
                },
                &[header, canvas],
            )
            .unwrap();
        Self {
            taffy,
            canvas,
            button,
            entry,
            root,
        }
    }

    pub fn compute(mut self, csize: Size) -> (Rect, Rect, Rect) {
        self.taffy
            .compute_layout(self.root, taffy::Size {
                width: taffy::AvailableSpace::Definite(csize.width as _),
                height: taffy::AvailableSpace::Definite(csize.height as _),
            })
            .unwrap();
        let entry_rect = self.taffy.layout(self.entry).unwrap();
        let button_rect = self.taffy.layout(self.button).unwrap();
        let canvas_rect = self.taffy.layout(self.canvas).unwrap();
        (
            rect_t2e(entry_rect),
            rect_t2e(button_rect),
            rect_t2e(canvas_rect),
        )
    }
}

fn rect_t2e(rect: &taffy::Layout) -> Rect {
    Rect::new(
        Point::new(rect.location.x as _, rect.location.y as _),
        Size::new(rect.size.width as _, rect.size.height as _),
    )
}
