use std::ops::Deref;

use winio::prelude::*;

pub struct WebViewPage {
    window: Child<TabViewItem>,
    button: Child<Button>,
    entry: Child<Edit>,
    webview: Child<WebView>,
}

#[derive(Debug)]
pub enum WebViewPageMessage {
    Noop,
    Go,
    Navigate,
}

impl Component for WebViewPage {
    type Event = ();
    type Init<'a> = &'a TabView;
    type Message = WebViewPageMessage;

    fn init(webview: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        let url = "https://www.example.com/";
        init! {
            window: TabViewItem = (webview) => {
                text: "WebView",
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

        sender.post(WebViewPageMessage::Go);

        Self {
            window,
            button,
            entry,
            webview,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: WebViewPageMessage::Noop,
            self.button => {
                ButtonEvent::Click => WebViewPageMessage::Go,
            },
            self.entry => {},
            self.webview => {
                WebViewEvent::Navigate => WebViewPageMessage::Navigate,
            }
        }
    }

    async fn update(&mut self, message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        futures_util::future::join4(
            self.window.update(),
            self.webview.update(),
            self.button.update(),
            self.entry.update(),
        )
        .await;
        match message {
            WebViewPageMessage::Noop => false,
            WebViewPageMessage::Go => {
                self.webview.navigate(self.entry.text());
                false
            }
            WebViewPageMessage::Navigate => {
                self.entry.set_text(self.webview.source());
                true
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.size();

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

impl Deref for WebViewPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
