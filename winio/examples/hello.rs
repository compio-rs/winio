use winio::prelude::*;

pub(crate) fn main() {
    #[cfg(all(feature = "enable_log", not(target_os = "android")))]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new("rs.compio.winio.widgets").run::<MainModel>(());
}

pub(crate) struct MainModel {
    window: Child<Window>,
    text: Child<Label>,
}

#[derive(Debug)]
pub(crate) enum MainMessage {
    Noop,
    Close,
    Redraw,
}

impl Component for MainModel {
    type Init<'a> = ();
    type Message = MainMessage;
    type Event = ();

    fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "Hello example",
                size: Size::new(800.0, 600.0),
            },
            text: Label = (&window) => {
                text: "Hello, world!",
                halign: HAlign::Center,
            },
        }

        window.show();

        Self { window, text }
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

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        futures_util::future::join(self.window.update(), self.text.update()).await;
        match message {
            MainMessage::Noop => false,
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Redraw => true,
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.client_size();
        {
            self.text.set_size(csize);
        }
    }
}
