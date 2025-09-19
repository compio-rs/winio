use winio::prelude::*;

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new("rs.compio.winio.net").run::<MainModel>("https://www.example.com/");
}

struct MainModel {
    button: Child<Button>,
    entry: Child<Edit>,
    webview: Child<WebView>,
    window: Child<Window>,
}

#[derive(Debug)]
enum MainMessage {
    Noop,
    Close,
    Redraw,
    Go,
    Navigate,
}

impl Component for MainModel {
    type Event = ();
    type Init<'a> = &'a str;
    type Message = MainMessage;

    fn init(url: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "Networking example",
                size: Size::new(800.0, 600.0),
            },
            webview: WebView = (&window) => {
                source: url
            },
            button: Button = (&window) => {
                text: "Go",
            },
            entry: Edit = (&window) => {
                text: url,
            },
        }

        sender.post(MainMessage::Go);

        window.show();

        Self {
            window,
            button,
            entry,
            webview,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.button => {
                ButtonEvent::Click => MainMessage::Go,
            },
            self.entry => {},
            self.webview => {
                WebViewEvent::Navigate => MainMessage::Navigate,
            }
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        futures_util::future::join4(
            self.window.update(),
            self.webview.update(),
            self.button.update(),
            self.entry.update(),
        )
        .await;
        match message {
            MainMessage::Noop => false,
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Redraw => true,
            MainMessage::Go => {
                self.webview.navigate(self.entry.text());
                false
            }
            MainMessage::Navigate => {
                self.entry.set_text(self.webview.source());
                true
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.client_size();

        {
            let mut header_panel = layout! {
                StackPanel::new(Orient::Horizontal),
                self.entry => { grow: true },
                self.button
            };
            let mut root_panel = layout! {
                StackPanel::new(Orient::Vertical),
                header_panel,
                self.webview => { grow: true },
            };
            root_panel.set_size(csize);
        }
    }
}
