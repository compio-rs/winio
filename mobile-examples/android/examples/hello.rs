#![no_main]

use winio::prelude::*;

#[unsafe(no_mangle)]
fn main() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("winio")
            .with_max_level(log::LevelFilter::Info),
    );
    App::new("rs.compio.winio.hello").run::<MainModel>(());
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
    type Init<'a> = ();
    type Message = MainMessage;
    type Event = ();

    fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "Hello example",
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
