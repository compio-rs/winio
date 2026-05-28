use winio::prelude::*;

#[cfg(target_os = "android")]
mod android;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error from the UI backend.
    #[error("UI error: {0}")]
    Ui(#[from] winio::Error),
    /// An error from [`winio_layout`].
    #[error("Layout error: {0}")]
    Layout(#[from] TaffyError),
    /// An IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl<E: Into<Error> + std::fmt::Display> From<LayoutError<E>> for Error {
    fn from(e: LayoutError<E>) -> Self {
        match e {
            LayoutError::Taffy(te) => Error::Layout(te),
            LayoutError::Child(ce) => ce.into(),
            _ => Error::Io(std::io::Error::other(e.to_string())),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct MainModel {
    window: Child<Window>,
    text: Child<Label>,
    link: Child<LinkLabel>,
}

#[derive(Debug)]
pub enum MainMessage {
    Noop,
    Close,
    Redraw,
}

impl Component for MainModel {
    type Error = Error;
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: Window = (()) => {
                text: "Hello example",
            },
            text: Label = (&window) => {
                text: "Hello, world!",
                halign: HAlign::Center,
            },
            link: LinkLabel = (&window) => {
                text: "Visit winio on GitHub",
                uri: "https://github.com/compio-rs/winio",
            },
        }

        window.show()?;

        Ok(Self { window, text, link })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.link => {},
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(self.window, self.text, self.link)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            MainMessage::Noop => Ok(false),
            MainMessage::Close => {
                sender.output(());
                Ok(false)
            }
            MainMessage::Redraw => Ok(true),
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.client_size()?;
        {
            let mut grid = layout! {
                Grid::from_str("1*, auto, 1*", "1*, auto, auto, 1*").unwrap(),
                self.text => { column: 1, row: 1 },
                self.link => { column: 1, row: 2 },
            };
            grid.set_size(csize)?;
        }
        Ok(())
    }
}
