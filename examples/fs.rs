use std::{io, path::Path};

use compio::{fs::File, io::AsyncReadAtExt, runtime::spawn};
use winio::{
    App, Button, ButtonEvent, Canvas, CanvasEvent, CanvasMessage, Child, Color, ColorTheme,
    Component, ComponentSender, DrawingFontBuilder, FileBox, HAlign, Point, Size, SolidColorBrush,
    VAlign, Window, WindowEvent,
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
    QueueRedraw,
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

        spawn(fetch(counter, sender.clone())).detach();

        Self {
            window,
            canvas,
            button,
            text: FetchStatus::Loading,
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
            ButtonEvent::Click => Some(MainMessage::ChooseFile),
            _ => None,
        });
        futures_util::future::join3(fut_window, fut_canvas, fut_button).await;
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
            MainMessage::QueueRedraw => self.canvas.emit(CanvasMessage::Redraw).await,
            MainMessage::Redraw => true,
            MainMessage::ChooseFile => {
                if let Some(p) = FileBox::new()
                    .title("Open file")
                    .add_filter(("All files", "*.*"))
                    .open(Some(&*self.window))
                    .await
                    .unwrap()
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
        const BHEIGHT: f64 = 20.0;

        let csize = self.window.client_size();

        self.button.set_loc(Point::new(0.0, 0.0));
        self.button.set_size(Size::new(csize.width, BHEIGHT));
        let bheight = self.button.size().height;

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
            },
        )
        .unwrap();
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
