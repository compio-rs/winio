use std::{
    io,
    ops::Deref,
    path::{Path, PathBuf},
};

use compio::{
    buf::buf_try,
    io::{AsyncReadAtExt, AsyncWriteAtExt},
    runtime::spawn,
};
use winio::prelude::*;

use crate::{Error, Result};

pub struct FsPage {
    window: Child<TabViewItem>,
    canvas: Child<Canvas>,
    open_button: Child<Button>,
    save_button: Child<Button>,
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
    SaveFile,
}

#[derive(Debug)]
pub enum FsPageMessage {
    Noop,
    ChooseFile,
    ChooseSaveFile,
    OpenFile(PathBuf),
    SaveFile(PathBuf),
    Fetch(FsFetchStatus),
}

impl Component for FsPage {
    type Error = Error;
    type Event = FsPageEvent;
    type Init<'a> = ();
    type Message = FsPageMessage;

    async fn init(_init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        let path = "Cargo.toml";
        init! {
            window: TabViewItem = (()) => {
                text: "File IO",
            },
            canvas: Canvas = (&window),
            open_button: Button = (&window) => {
                text: "Choose file...",
            },
            save_button: Button = (&window) => {
                text: "Save file...",
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
            open_button,
            save_button,
            label,
            text: FsFetchStatus::Loading,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: FsPageMessage::Noop,
            self.open_button => {
                ButtonEvent::Click => FsPageMessage::ChooseFile,
            },
            self.save_button => {
                ButtonEvent::Click => FsPageMessage::ChooseSaveFile,
            },
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(
            self.window,
            self.canvas,
            self.open_button,
            self.save_button,
            self.label
        )
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
            FsPageMessage::ChooseSaveFile => {
                sender.output(FsPageEvent::SaveFile);
                Ok(false)
            }
            FsPageMessage::OpenFile(p) => {
                self.label.set_text(p.to_str().unwrap_or_default())?;
                spawn(fetch(p, sender.clone())).detach();
                Ok(true)
            }
            FsPageMessage::SaveFile(p) => {
                let data = match &self.text {
                    FsFetchStatus::Complete(s) => s.clone(),
                    _ => String::new(),
                };
                spawn(save(p, data, sender.clone())).detach();
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
                self.label, self.open_button, self.save_button,
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

async fn read_file(path: impl AsRef<Path>) -> Result<String> {
    let file = UriFile::open(path).await?;
    let (_, buffer) = buf_try!(@try file.read_to_end_at(vec![], 0).await);
    Ok(String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
}

async fn fetch(path: impl AsRef<Path>, sender: ComponentSender<FsPage>) {
    sender.post(FsPageMessage::Fetch(FsFetchStatus::Loading));
    let status = match read_file(path).await {
        Ok(text) => FsFetchStatus::Complete(text),
        Err(e) => FsFetchStatus::Error(format!("{e:?}")),
    };
    sender.post(FsPageMessage::Fetch(status));
}

async fn write_file(path: impl AsRef<Path>, data: String) -> Result<()> {
    let mut file = UriFile::create(path).await?;
    file.write_all_at(data, 0).await.0?;
    Ok(())
}

async fn save(path: impl AsRef<Path>, data: String, sender: ComponentSender<FsPage>) {
    match write_file(path, data).await {
        Ok(()) => {}
        Err(e) => {
            sender.post(FsPageMessage::Fetch(FsFetchStatus::Error(format!("{e:?}"))));
        }
    }
}
