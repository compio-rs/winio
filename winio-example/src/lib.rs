#![cfg(target_os = "android")]

use android_activity::AndroidApp;
use compio_log::metadata::LevelFilter;
use tracing_subscriber::prelude::*;
use winio::prelude::*;

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

type Result<T> = std::result::Result<T, Error>;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    tracing_subscriber::registry()
        .with(tracing_android_trace::AndroidTraceLayer::new())
        .with(tracing_subscriber::fmt::layer().with_filter(LevelFilter::TRACE))
        .init();

    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if let Err(e) = App::new("rs.compio.winio.hello", app)
        .expect("cannot create app")
        .run_until_event::<MainModel>(())
    {
        compio_log::error!("App error: {e:?}");
    }
}

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
                uri: "https://github.com/compio-rs/winio.git",
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
        compio_log::info!("csize: {csize:?}");
        {
            let mut panel = layout! {
                StackPanel::new(Orient::Vertical),
                self.text => { margin: Margin::new_all_same(4.0), halign: HAlign::Center },
                self.link => { margin: Margin::new_all_same(4.0), halign: HAlign::Center },
            };
            panel.set_size(csize)?;
        }
        compio_log::info!(
            "text rect: {:?}, link rect: {:?}",
            self.text.rect()?,
            self.link.rect()?
        );
        Ok(())
    }
}
