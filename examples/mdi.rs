use winio::{
    App, Child, Component, ComponentSender, Layoutable, Size, Visible, Window, WindowEvent, init,
    start,
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new().run::<MainModel>(());
}

struct MainModel {
    window: Child<Window>,
    cwindow: Child<Window>,
}

#[derive(Debug)]
enum MainMessage {
    Noop,
    Close,
    Redraw,
}

impl Component for MainModel {
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "MDI example",
                size: Size::new(800.0, 600.0),
            },
            cwindow: Window = (&window) => {
                text: "Child window",
                size: Size::new(400.0, 300.0),
            }
        }

        cwindow.show();
        window.show();

        Self { window, cwindow }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.cwindow => {
                WindowEvent::Resize => MainMessage::Redraw,
            }
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        futures_util::future::join(self.window.update(), self.cwindow.update()).await;
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
        self.window.render();
        self.cwindow.render();
    }
}
