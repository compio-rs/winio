use std::ops::Deref;

use winio::prelude::*;

use crate::{Error, Result};

pub struct ClipboardPage {
    window: Child<TabViewItem>,
    clipboard: Child<Clipboard>,
    clipboard_group: Child<GroupBox>,
    clipboard_text: Child<TextBox>,
    clipboard_picture: Child<Picture>,
    clipboard_image: Option<BitmapSize>,
    drop_group: Child<GroupBox>,
    drop_area: Child<DropArea>,
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
            clipboard_text: TextBox = (&clipboard_group) => {
                readonly: true,
            },
            clipboard_picture: Picture = (&clipboard_group),
            drop_group: GroupBox = (&window) => {
                text: "Drop",
            },
            drop_area: DropArea = (&drop_group),
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
            clipboard_text,
            clipboard_picture,
            clipboard_image: None,
            drop_group,
            drop_area,
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
            self.clipboard_text => {},
            self.clipboard_picture => {},
            self.drop_group => {},
            self.drop_area => {
                DropAreaEvent::Drop(data) => ClipboardPageMessage::Drop(data),
            },
            self.drop_text => {},
            self.drop_picture => {},
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(
            self.window,
            self.clipboard,
            self.clipboard_group,
            self.clipboard_text,
            self.clipboard_picture,
            self.drop_group,
            self.drop_area,
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
                self.clipboard_image = match self.clipboard.read().await? {
                    Some(data) => Self::show_data(
                        data,
                        &mut self.clipboard_text,
                        &mut self.clipboard_picture,
                    )?,
                    None => {
                        self.clipboard_text.set_text("")?;
                        self.clipboard_picture.set_image(None)?;
                        None
                    }
                };
                Ok(true)
            }
            ClipboardPageMessage::Drop(data) => {
                self.drop_image =
                    Self::show_data(data, &mut self.drop_text, &mut self.drop_picture)?;
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
            &mut self.clipboard_text,
            &mut self.clipboard_picture,
            self.clipboard_image,
            clipboard_size,
        );
        clipboard_panel.set_size(clipboard_size)?;

        let drop_size = self.drop_group.client_size()?;
        self.drop_area.set_rect(drop_size.into())?;
        let mut drop_panel = Self::value_panel(
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
        text: &'a mut Child<TextBox>,
        picture: &'a mut Child<Picture>,
        image: Option<BitmapSize>,
        size: Size,
    ) -> Grid<'a, winio::Error> {
        let picture_size = image
            .map(|image| Self::fit_picture(image, size))
            .unwrap_or_else(Size::zero);
        layout! {
            Grid::from_str("1*,1*", "1*").unwrap(),
            *text => {
                margin: Margin::new_all_same(4.0),
                column: 0,
                row: 0,
            },
            *picture => {
                column: 1,
                row: 0,
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

    fn show_data(
        data: Data,
        text: &mut Child<TextBox>,
        picture: &mut Child<Picture>,
    ) -> Result<Option<BitmapSize>> {
        let mut formats = data.formats()?.collect::<Vec<_>>();
        formats.sort_unstable_by_key(|format| match format {
            DataFormat::Image => 0,
            DataFormat::Paths => 1,
            DataFormat::Html => 2,
            DataFormat::Text => 3,
            _ => 4,
        });

        let Some(format) = formats.into_iter().next() else {
            return Self::show_text(text, picture, String::new());
        };
        let Some(entry) = data.into_format(format).next() else {
            return Self::show_text(text, picture, String::new());
        };
        match entry {
            Ok(entry) => Self::show_entry(entry, text, picture),
            Err(_) => Self::show_text(text, picture, String::new()),
        }
    }

    fn show_entry(
        entry: DataEntry<Image>,
        text: &mut Child<TextBox>,
        picture: &mut Child<Picture>,
    ) -> Result<Option<BitmapSize>> {
        match entry {
            DataEntry::Image(image) => {
                let size = image.size()?;
                text.set_text(format!("{:?}", size))?;
                picture.set_image(Some(&image))?;
                Ok(Some(size))
            }
            DataEntry::Paths(paths) => {
                let body = paths
                    .iter()
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("\n");
                Self::show_text(text, picture, body)
            }
            DataEntry::Html(html) => Self::show_text(text, picture, html),
            DataEntry::Text(body) => Self::show_text(text, picture, body),
            DataEntry::Unknown(bytes) => {
                Self::show_text(text, picture, format!("{} bytes", bytes.len()))
            }
        }
    }

    fn show_text(
        text: &mut Child<TextBox>,
        picture: &mut Child<Picture>,
        body: String,
    ) -> Result<Option<BitmapSize>> {
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
