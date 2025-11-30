use std::ops::Deref;

use winio::prelude::*;

use crate::{Error, Result};

pub struct WebViewPage {
    window: Child<TabViewItem>,
    go_button: Child<Button>,
    back_button: Child<Button>,
    forward_button: Child<Button>,
    reload_button: Child<Button>,
    can_reload: bool,
    entry: Child<Edit>,
    webview: Child<WebView>,
}

impl WebViewPage {
    fn set_reload_button(&mut self, enabled: bool) -> Result<()> {
        self.back_button.set_enabled(self.webview.can_go_back()?)?;
        self.forward_button
            .set_enabled(self.webview.can_go_forward()?)?;

        self.can_reload = enabled;
        if enabled {
            self.reload_button.set_text("🔄")?;
        } else {
            self.reload_button.set_text("⏹️")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum WebViewPageEvent {}

#[derive(Debug)]
pub enum WebViewPageMessage {
    Noop,
    Go,
    Back,
    Forward,
    Reload,
    Navigating,
    Navigated,
}

impl Component for WebViewPage {
    type Error = Error;
    type Event = WebViewPageEvent;
    type Init<'a> = &'a TabView;
    type Message = WebViewPageMessage;

    async fn init(webview: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        let url = "https://www.example.com/";
        init! {
            window: TabViewItem = (webview) => {
                text: "WebView",
            },
            webview: WebView = (&window) => {
                source: url
            },
            go_button: Button = (&window) => {
                text: "⬇️",
            },
            back_button: Button = (&window) => {
                text: "⬅️",
                enabled: false,
            },
            forward_button: Button = (&window) => {
                text: "➡️",
                enabled: false,
            },
            reload_button: Button = (&window) => {
                text: "🔄",
            },
            entry: Edit = (&window) => {
                text: url,
            },
        }

        sender.post(WebViewPageMessage::Go);

        Ok(Self {
            window,
            go_button,
            back_button,
            forward_button,
            reload_button,
            can_reload: true,
            entry,
            webview,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: WebViewPageMessage::Noop,
            self.go_button => {
                ButtonEvent::Click => WebViewPageMessage::Go,
            },
            self.back_button => {
                ButtonEvent::Click => WebViewPageMessage::Back,
            },
            self.forward_button => {
                ButtonEvent::Click => WebViewPageMessage::Forward,
            },
            self.reload_button => {
                ButtonEvent::Click => WebViewPageMessage::Reload,
            },
            self.entry => {},
            self.webview => {
                WebViewEvent::Navigating => WebViewPageMessage::Navigating,
                WebViewEvent::Navigated => WebViewPageMessage::Navigated,
            }
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(
            self.window,
            self.webview,
            self.go_button,
            self.back_button,
            self.forward_button,
            self.reload_button,
            self.entry
        )
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            WebViewPageMessage::Noop => Ok(false),
            WebViewPageMessage::Go => {
                self.webview.navigate(self.entry.text()?)?;
                self.set_reload_button(false)?;
                Ok(false)
            }
            WebViewPageMessage::Back => {
                self.webview.go_back()?;
                self.set_reload_button(false)?;
                Ok(false)
            }
            WebViewPageMessage::Forward => {
                self.webview.go_forward()?;
                self.set_reload_button(false)?;
                Ok(false)
            }
            WebViewPageMessage::Reload => {
                if self.can_reload {
                    self.webview.reload()?;
                    self.set_reload_button(false)?;
                } else {
                    self.webview.stop()?;
                    self.set_reload_button(true)?;
                }
                Ok(false)
            }
            WebViewPageMessage::Navigating => {
                self.entry.set_text(self.webview.source()?)?;
                self.set_reload_button(false)?;
                Ok(false)
            }
            WebViewPageMessage::Navigated => {
                self.entry.set_text(self.webview.source()?)?;
                self.set_reload_button(true)?;
                Ok(true)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.size()?;

        {
            let mut header_panel = layout! {
                StackPanel::new(Orient::Horizontal),
                self.back_button,
                self.forward_button,
                self.reload_button,
                self.entry => { grow: true },
                self.go_button,
            };
            let mut root_panel = layout! {
                StackPanel::new(Orient::Vertical),
                header_panel,
                self.webview => { grow: true },
            };
            root_panel.set_size(csize)?;
        }
        Ok(())
    }
}

impl Deref for WebViewPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
