use std::{
    io,
    ops::Deref,
    path::{Path, PathBuf},
};

use compio::{buf::buf_try, fs::File, io::AsyncReadAtExt, runtime::spawn};
use tuplex::IntoArray;
use winio::prelude::*;

use crate::{Error, Result};

pub struct FsPage {
    window: Child<TabViewItem>,
    canvas: Child<Canvas>,
    button: Child<Button>,
    label: Child<Label>,
    text: FsFetchStatus,
}

#[derive(Debug)]
pub enum FsFetchStatus {
    Loading,
    Complete(String),
    Error(String),
}

#[derive(Debug)]
pub enum FsPageEvent {
    ChooseFile,
}

#[derive(Debug)]
pub enum FsPageMessage {
    Noop,
    ChooseFile,
    OpenFile(PathBuf),
    Fetch(FsFetchStatus),
}

impl Failable for FsPage {
    type Error = Error;
}

impl Component for FsPage {
    type Event = FsPageEvent;
    type Init<'a> = &'a TabView;
    type Message = FsPageMessage;

    fn init(tabview: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        let path = "Cargo.toml";
        init! {
            window: TabViewItem = (tabview) => {
                text: "File IO",
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

        Ok(Self {
            window,
            canvas,
            button,
            label,
            text: FsFetchStatus::Loading,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: FsPageMessage::Noop,
            self.button => {
                ButtonEvent::Click => FsPageMessage::ChooseFile,
            },
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        Ok(futures_util::future::try_join4(
            self.window.update(),
            self.canvas.update(),
            self.button.update(),
            self.label.update(),
        )
        .await?
        .into_array()
        .into_iter()
        .any(|b| b))
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            FsPageMessage::Noop => Ok(false),
            FsPageMessage::ChooseFile => {
                sender.output(FsPageEvent::ChooseFile);
                Ok(false)
            }
            FsPageMessage::OpenFile(p) => {
                self.label.set_text(p.to_str().unwrap_or_default());
                spawn(fetch(p, sender.clone())).detach();
                Ok(true)
            }
            FsPageMessage::Fetch(status) => {
                self.text = status;
                Ok(true)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.size()?;

        {
            let mut panel = layout! {
                StackPanel::new(Orient::Vertical),
                self.label, self.button,
                self.canvas => { grow: true }
            };
            panel.set_size(csize)?;
        }

        let mut ctx = self.canvas.context()?;
        let is_dark = ColorTheme::current()? == ColorTheme::Dark;
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
                FsFetchStatus::Loading => "Loading...",
                FsFetchStatus::Complete(s) => s.as_str(),
                FsFetchStatus::Error(e) => e.as_str(),
            },
        )?;
        Ok(())
    }
}

impl Deref for FsPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

async fn read_file(path: impl AsRef<Path>) -> std::io::Result<String> {
    let file = File::open(path).await?;
    let (_, buffer) = buf_try!(@try file.read_to_end_at(vec![], 0).await);
    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

async fn fetch(path: impl AsRef<Path>, sender: ComponentSender<FsPage>) {
    sender.post(FsPageMessage::Fetch(FsFetchStatus::Loading));
    let status = match read_file(path).await {
        Ok(text) => FsFetchStatus::Complete(text),
        Err(e) => FsFetchStatus::Error(format!("{e:?}")),
    };
    sender.post(FsPageMessage::Fetch(status));
}
