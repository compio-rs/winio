use std::{io, path::Path};

use compio::{fs::File, io::AsyncReadAtExt, runtime::spawn};
use taffy::{NodeId, TaffyTree};
use winio::{
    App, Button, ButtonEvent, Canvas, Child, Color, ColorTheme, Component, ComponentSender,
    DrawingFontBuilder, FileBox, HAlign, Point, Rect, Size, SolidColorBrush, VAlign, Window,
    WindowEvent,
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new().run::<MainModel>("Cargo.toml", &())
}

struct MainModel {
    window: Child<Window>,
    canvas: Child<Canvas>,
    button: Child<Button>,
    text: FetchStatus,
    bheight: f64,
    is_dark: bool,
}

#[derive(Debug)]
enum FetchStatus {
    Loading,
    Complete(String),
    Error(String),
}

#[derive(Debug)]
enum MainMessage {
    Close,
    Redraw,
    ChooseFile,
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
        window.set_text("File IO example");
        window.set_size(Size::new(800.0, 600.0));

        let is_dark = ColorTheme::current() == ColorTheme::Dark;

        let canvas = Child::<Canvas>::init((), &window);
        let mut button = Child::<Button>::init((), &window);
        button.set_text("Choose file...");
        button.set_loc(Point::zero());
        button.set_size(Size::new(800.0, 20.0));
        let bheight = button.size().height;

        spawn(fetch(counter, sender.clone())).detach();

        Self {
            window,
            canvas,
            button,
            text: FetchStatus::Loading,
            bheight,
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
            ButtonEvent::Click => Some(MainMessage::ChooseFile),
            _ => None,
        });
        futures_util::future::join(fut_window, fut_button).await;
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &winio::ComponentSender<Self>,
    ) -> bool {
        futures_util::future::join3(
            self.window.update(),
            self.canvas.update(),
            self.button.update(),
        )
        .await;
        match message {
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Redraw => true,
            MainMessage::ChooseFile => {
                if let Some(p) = FileBox::new()
                    .title("Open file")
                    .add_filter(("All files", "*.*"))
                    .open(Some(&*self.window))
                    .await
                {
                    spawn(fetch(p, sender.clone())).detach();
                }
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

        let (brect, crect) = Layout::new(self.bheight).compute(csize);
        self.button.set_size(brect.size);
        self.canvas.set_loc(crect.origin);
        self.canvas.set_size(crect.size);

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
            },
        );
    }
}

async fn read_file(path: impl AsRef<Path>) -> io::Result<String> {
    let file = File::open(path).await?;
    let (_, buffer) = file.read_to_end_at(vec![], 0).await?;
    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

async fn fetch(path: impl AsRef<Path>, sender: ComponentSender<MainModel>) {
    sender.post(MainMessage::Fetch(FetchStatus::Loading));
    let status = match read_file(path).await {
        Ok(text) => FetchStatus::Complete(text),
        Err(e) => FetchStatus::Error(format!("{:?}", e)),
    };
    sender.post(MainMessage::Fetch(status));
}

struct Layout {
    taffy: TaffyTree,
    canvas: NodeId,
    button: NodeId,
    root: NodeId,
}

impl Layout {
    pub fn new(bheight: f64) -> Self {
        let mut taffy = TaffyTree::new();
        let button = taffy
            .new_leaf(taffy::Style {
                size: taffy::Size {
                    width: taffy::Dimension::Percent(1.0),
                    height: taffy::Dimension::Length(bheight as _),
                },
                ..Default::default()
            })
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
                &[button, canvas],
            )
            .unwrap();
        Self {
            taffy,
            canvas,
            button,
            root,
        }
    }

    pub fn compute(mut self, csize: Size) -> (Rect, Rect) {
        self.taffy
            .compute_layout(self.root, taffy::Size {
                width: taffy::AvailableSpace::Definite(csize.width as _),
                height: taffy::AvailableSpace::Definite(csize.height as _),
            })
            .unwrap();
        let button_rect = self.taffy.layout(self.button).unwrap();
        let canvas_rect = self.taffy.layout(self.canvas).unwrap();
        (rect_t2e(button_rect), rect_t2e(canvas_rect))
    }
}

fn rect_t2e(rect: &taffy::Layout) -> Rect {
    Rect::new(
        Point::new(rect.location.x as _, rect.location.y as _),
        Size::new(rect.size.width as _, rect.size.height as _),
    )
}
