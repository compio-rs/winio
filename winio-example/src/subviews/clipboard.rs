use std::ops::Deref;

use winio::prelude::*;

use crate::{Error, Result};

pub struct ClipboardPage {
    window: Child<TabViewItem>,
    clipboard: Child<Clipboard>,
    clipboard_group: Child<GroupBox>,
    clipboard_mime: Child<Label>,
    clipboard_text: Child<TextBox>,
    clipboard_picture: Child<Picture>,
    clipboard_image: Option<BitmapSize>,
    drop_group: Child<GroupBox>,
    drop_area: Child<DropArea>,
    drop_mime: Child<Label>,
    drop_text: Child<TextBox>,
    drop_picture: Child<Picture>,
    drop_image: Option<BitmapSize>,
}

#[derive(Debug)]
pub enum ClipboardPageEvent {}

#[derive(Debug)]
pub enum ClipboardPageMessage {
    Refresh,
    Drop(Data),
}

impl Component for ClipboardPage {
    type Error = Error;
    type Event = ClipboardPageEvent;
    type Init<'a> = &'a TabView;
    type Message = ClipboardPageMessage;

    async fn init(init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: TabViewItem = (init) => {
                text: "Clipboard",
            },
            clipboard: Clipboard = (()),
            clipboard_group: GroupBox = (&window) => {
                text: "Clipboard",
            },
            clipboard_mime: Label = (&clipboard_group) => {
                text: "(empty)",
            },
            clipboard_text: TextBox = (&clipboard_group) => {
                readonly: true,
            },
            clipboard_picture: Picture = (&clipboard_group),
            drop_group: GroupBox = (&window) => {
                text: "Drop",
            },
            drop_area: DropArea = (&drop_group),
            drop_mime: Label = (&drop_area) => {
                text: "(no drop yet)",
            },
            drop_text: TextBox = (&drop_area) => {
                readonly: true,
            },
            drop_picture: Picture = (&drop_area),
        }

        sender.post(ClipboardPageMessage::Refresh);

        Ok(Self {
            window,
            clipboard,
            clipboard_group,
            clipboard_mime,
            clipboard_text,
            clipboard_picture,
            clipboard_image: None,
            drop_group,
            drop_area,
            drop_mime,
            drop_text,
            drop_picture,
            drop_image: None,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender,
            self.window => {},
            self.clipboard => {
                ClipboardEvent::Changed => ClipboardPageMessage::Refresh,
            },
            self.clipboard_group => {},
            self.clipboard_mime => {},
            self.clipboard_text => {},
            self.clipboard_picture => {},
            self.drop_group => {},
            self.drop_area => {
                DropAreaEvent::Drop(data) => ClipboardPageMessage::Drop(data),
            },
            self.drop_mime => {},
            self.drop_text => {},
            self.drop_picture => {},
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(
            self.window,
            self.clipboard,
            self.clipboard_group,
            self.clipboard_mime,
            self.clipboard_text,
            self.clipboard_picture,
            self.drop_group,
            self.drop_area,
            self.drop_mime,
            self.drop_text,
            self.drop_picture,
        )
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ClipboardPageMessage::Refresh => {
                self.clipboard_image = match self.clipboard.read()? {
                    Some(data) => Self::show_known(
                        data.into_known(),
                        &mut self.clipboard_mime,
                        &mut self.clipboard_text,
                        &mut self.clipboard_picture,
                    )?,
                    None => {
                        self.clipboard_mime.set_text("(empty)")?;
                        self.clipboard_text.set_text("")?;
                        self.clipboard_picture.set_image(None)?;
                        None
                    }
                };
                Ok(true)
            }
            ClipboardPageMessage::Drop(data) => {
                self.drop_image = Self::show_known(
                    data.into_known(),
                    &mut self.drop_mime,
                    &mut self.drop_text,
                    &mut self.drop_picture,
                )?;
                Ok(true)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let size = self.window.size()?;
        let mut root = layout! {
            StackPanel::new(Orient::Vertical),
            self.clipboard_group => {
                margin: Margin::new_all_same(8.0),
                grow: true,
            },
            self.drop_group => {
                margin: Margin::new_all_same(8.0),
                grow: true,
            },
        };
        root.set_size(size)?;

        let clipboard_size = self.clipboard_group.client_size()?;
        let mut clipboard_panel = Self::value_panel(
            &mut self.clipboard_mime,
            &mut self.clipboard_text,
            &mut self.clipboard_picture,
            self.clipboard_image,
            clipboard_size,
        );
        clipboard_panel.set_size(clipboard_size)?;

        let drop_size = self.drop_group.client_size()?;
        self.drop_area.set_rect(drop_size.into())?;
        let mut drop_panel = Self::value_panel(
            &mut self.drop_mime,
            &mut self.drop_text,
            &mut self.drop_picture,
            self.drop_image,
            drop_size,
        );
        drop_panel.set_size(drop_size)?;

        Ok(())
    }
}

impl ClipboardPage {
    fn value_panel<'a>(
        mime: &'a mut Child<Label>,
        text: &'a mut Child<TextBox>,
        picture: &'a mut Child<Picture>,
        image: Option<BitmapSize>,
        size: Size,
    ) -> Grid<'a, winio::Error> {
        let picture_size = image
            .map(|image| Self::fit_picture(image, size))
            .unwrap_or_else(Size::zero);
        layout! {
            Grid::from_str("1*,1*", "auto,1*").unwrap(),
            *mime => {
                margin: Margin::new_all_same(4.0),
                column: 0,
                row: 0,
            },
            *text => {
                margin: Margin::new_all_same(4.0),
                column: 0,
                row: 1,
            },
            *picture => {
                column: 1,
                row: 1,
                width: picture_size.width,
                height: picture_size.height,
                min_width: 0.0,
                min_height: 0.0,
                halign: HAlign::Center,
                valign: VAlign::Center,
            },
        }
    }

    fn fit_picture(image: BitmapSize, group: Size) -> Size {
        let max_width = (group.width / 2.0 - 8.0).max(0.0);
        let max_height = (group.height - 8.0).max(0.0);
        let aspect = image.width as f64 / image.height as f64;
        let mut height = max_height;
        let mut width = height * aspect;
        if width > max_width {
            width = max_width;
            height = width / aspect;
        }
        Size::new(width, height)
    }

    fn show_known(
        known: std::result::Result<KnownData, Data>,
        mime: &mut Child<Label>,
        text: &mut Child<TextBox>,
        picture: &mut Child<Picture>,
    ) -> Result<Option<BitmapSize>> {
        match known {
            Ok(KnownData::Text(body)) => Self::show_text(mime, text, picture, "text/plain", body),
            Ok(KnownData::Html(body)) => Self::show_text(mime, text, picture, "text/html", body),
            Ok(KnownData::Rtf(body)) => Self::show_text(mime, text, picture, "text/rtf", body),
            Ok(KnownData::Uris(uris)) => {
                let body = uris
                    .iter()
                    .map(|uri| uri.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("\n");
                Self::show_text(mime, text, picture, "text/uri-list", body)
            }
            Ok(KnownData::Image(image)) => {
                let size = image.size()?;
                mime.set_text("image/png")?;
                text.set_text(format!("{:?}", size))?;
                picture.set_image(Some(&image))?;
                Ok(Some(size))
            }
            Ok(_) => {
                mime.set_text("(unknown)")?;
                text.set_text("")?;
                picture.set_image(None)?;
                Ok(None)
            }
            Err(data) => {
                let formats = data
                    .formats()?
                    .iter()
                    .map(|mime| mime.essence_str().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                mime.set_text(if formats.is_empty() {
                    "(unknown)"
                } else {
                    formats.as_str()
                })?;
                text.set_text("")?;
                picture.set_image(None)?;
                Ok(None)
            }
        }
    }

    fn show_text(
        mime: &mut Child<Label>,
        text: &mut Child<TextBox>,
        picture: &mut Child<Picture>,
        mime_text: &str,
        body: String,
    ) -> Result<Option<BitmapSize>> {
        mime.set_text(mime_text)?;
        text.set_text(body)?;
        picture.set_image(None)?;
        Ok(None)
    }
}

impl Deref for ClipboardPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
