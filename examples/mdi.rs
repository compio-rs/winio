use winio::{
    App, Child, Component, ComponentSender, Layoutable, Size, Visible, Window, WindowEvent,
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
        let mut window = Child::<Window>::init(());
        let mut cwindow = Child::<Window>::init(&window);

        window.set_text("MDI example");
        window.set_size(Size::new(800.0, 600.0));

        cwindow.set_text("Child window");
        cwindow.set_size(Size::new(400.0, 300.0));

        cwindow.show();
        window.show();

        Self { window, cwindow }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        let fut_window = self.window.start(
            sender,
            |e| match e {
                WindowEvent::Close => Some(MainMessage::Close),
                WindowEvent::Resize => Some(MainMessage::Redraw),
                _ => None,
            },
            || MainMessage::Noop,
        );
        let fut_cwindow = self.cwindow.start(
            sender,
            |e| match e {
                WindowEvent::Resize => Some(MainMessage::Redraw),
                _ => None,
            },
            || MainMessage::Noop,
        );
        futures_util::future::join(fut_window, fut_cwindow).await;
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
