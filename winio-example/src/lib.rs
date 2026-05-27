#![cfg(target_os = "android")]

use android_activity::AndroidApp;
use compio_log::metadata::LevelFilter;
use tracing_subscriber::prelude::*;
use winio::{Error, Result, prelude::*};

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
        .run::<MainModel>(())
    {
        compio_log::error!("App error: {e:?}");
    }
}

pub struct MainModel {
    window: Child<Window>,
    text: Child<Label>,
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
        }

        window.show()?;

        Ok(Self { window, text })
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
        update_children!(self.window, self.text)
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
            self.text.set_size(csize)?;
        }
        Ok(())
    }
}
