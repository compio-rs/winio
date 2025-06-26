use winio::{
    App, CheckBox, CheckBoxEvent, Child, Component, ComponentSender, Layoutable, Size, Visible,
    Window, WindowEvent, init, start,
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new("rs.compio.winio.mdi").run::<MainModel>(());
}

struct MainModel {
    window: Child<Window>,
    cwindow: Child<ChildModel>,
}

#[derive(Debug)]
enum MainMessage {
    Noop,
    Close,
    Redraw,
    Check(bool),
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
            cwindow: ChildModel = (&*window),
        }

        window.show();

        Self { window, cwindow }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.cwindow => {
                ChildEvent::Check(b) => MainMessage::Check(b),
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
            MainMessage::Check(b) => {
                self.window
                    .set_text(if b { "Checked" } else { "MDI example" });
                false
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.window.render();
        self.cwindow.render();
    }
}

struct ChildModel {
    window: Child<Window>,
    check: Child<CheckBox>,
}

#[derive(Debug)]
enum ChildMessage {
    Noop,
    Redraw,
    Check,
}

#[derive(Debug)]
enum ChildEvent {
    Check(bool),
}

impl Component for ChildModel {
    type Event = ChildEvent;
    type Init<'a> = &'a Window;
    type Message = ChildMessage;

    fn init(root: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (root) => {
                text: "Child window",
                size: Size::new(400.0, 300.0),
            },
            check: CheckBox = (&window) => {
                text: "Check me",
            },
        }

        window.show();

        Self { window, check }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: ChildMessage::Noop,
            self.window => {
                WindowEvent::Resize => ChildMessage::Redraw,
            },
            self.check => {
                CheckBoxEvent::Click => ChildMessage::Check,
            }
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        futures_util::future::join(self.window.update(), self.check.update()).await;
        match message {
            ChildMessage::Noop => false,
            ChildMessage::Redraw => true,
            ChildMessage::Check => {
                sender.output(ChildEvent::Check(self.check.is_checked()));
                true
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.window.render();
        self.check.render();

        let csize = self.window.client_size();
        let psize = self.check.preferred_size();
        self.check.set_size(Size::new(csize.width, psize.height));
    }
}
