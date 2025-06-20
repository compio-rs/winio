use std::{
    io,
    path::{Path, PathBuf},
};

use compio::{fs::File, io::AsyncReadAtExt, runtime::spawn};
use winio::{
    App, Button, ButtonEvent, Canvas, CanvasEvent, Child, Color, ColorTheme, Component,
    ComponentSender, DrawingFontBuilder, FileBox, HAlign, Label, Layoutable, Orient, Point, Size,
    SolidColorBrush, StackPanel, VAlign, Visible, Window, WindowEvent, init, layout, start,
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new().run::<MainModel>("Cargo.toml")
}

struct MainModel {
    window: Child<Window>,
    canvas: Child<Canvas>,
    button: Child<Button>,
    label: Child<Label>,
    text: FetchStatus,
}

#[derive(Debug)]
enum FetchStatus {
    Loading,
    Complete(String),
    Error(String),
}

#[derive(Debug)]
enum MainMessage {
    Noop,
    Close,
    Redraw,
    ChooseFile,
    OpenFile(PathBuf),
    Fetch(FetchStatus),
}

impl Component for MainModel {
    type Event = ();
    type Init<'a> = &'a str;
    type Message = MainMessage;

    fn init(path: Self::Init<'_>, sender: &winio::ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "File IO example",
                size: Size::new(800.0, 600.0),
            },
            canvas: Canvas = (&window),
            button: Button = (&window) => {
                text: "Choose file...",
            },
            label: Label = (&window) => {
                text: path,
                halign: HAlign::Center,
            },
        }

        let path = path.to_string();
        spawn(fetch(path, sender.clone())).detach();

        window.show();

        Self {
            window,
            canvas,
            button,
            label,
            text: FetchStatus::Loading,
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
                ButtonEvent::Click => MainMessage::ChooseFile,
            },
        }
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
            MainMessage::Noop => false,
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Redraw => true,
            MainMessage::ChooseFile => {
                if let Some(p) = FileBox::new()
                    .title("Open file")
                    .add_filter(("All files", "*.*"))
                    .open(&self.window)
                    .await
                {
                    sender.post(MainMessage::OpenFile(p));
                }
                false
            }
            MainMessage::OpenFile(p) => {
                self.label.set_text(p.to_str().unwrap_or_default());
                spawn(fetch(p, sender.clone())).detach();
                true
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
            let mut panel = layout! {
                StackPanel::new(Orient::Vertical),
                self.label, self.button,
                self.canvas => { grow: true }
            };
            panel.set_size(csize);
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
        Err(e) => FetchStatus::Error(format!("{e:?}")),
    };
    sender.post(MainMessage::Fetch(status));
}
