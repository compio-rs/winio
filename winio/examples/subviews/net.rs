use std::{ops::Deref, time::Duration};

use compio::{runtime::spawn, time::timeout};
use cyper::Client;
use tuplex::IntoArray;
use winio::prelude::*;

use crate::{Error, Result};

pub struct NetPage {
    window: Child<TabViewItem>,
    canvas: Child<Canvas>,
    button: Child<Button>,
    entry: Child<Edit>,
    client: Client,
    text: NetFetchStatus,
}

#[derive(Debug)]
pub enum NetFetchStatus {
    Loading,
    Complete(String),
    Error(String),
    Timedout,
}

#[derive(Debug)]
pub enum NetPageMessage {
    Noop,
    Go,
    Fetch(NetFetchStatus),
}

impl Failable for NetPage {
    type Error = Error;
}

impl Component for NetPage {
    type Event = ();
    type Init<'a> = &'a TabView;
    type Message = NetPageMessage;

    fn init(tabview: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        let url = "https://www.example.com/";
        init! {
            window: TabViewItem = (tabview) => {
                text: "Networking",
            },
            canvas: Canvas = (&window),
            button: Button = (&window) => {
                text: "Go",
            },
            entry: Edit = (&window) => {
                text: url,
            },
        }

        let client = Client::new();

        let url = url.to_string();
        spawn(fetch(client.clone(), url, sender.clone())).detach();

        Ok(Self {
            window,
            canvas,
            button,
            entry,
            text: NetFetchStatus::Loading,
            client,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: NetPageMessage::Noop,
            self.button => {
                ButtonEvent::Click => NetPageMessage::Go,
            },
            self.entry => {},
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        Ok(futures_util::future::try_join4(
            self.window.update(),
            self.canvas.update(),
            self.entry.update(),
            self.button.update(),
        )
        .await?
        .into_array()
        .into_iter()
        .any(|b| b))
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            NetPageMessage::Noop => Ok(false),
            NetPageMessage::Go => {
                spawn(fetch(
                    self.client.clone(),
                    self.entry.text()?,
                    sender.clone(),
                ))
                .detach();
                Ok(false)
            }
            NetPageMessage::Fetch(status) => {
                self.text = status;
                Ok(true)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.size()?;

        {
            let mut header_panel = layout! {
                StackPanel::new(Orient::Horizontal),
                self.entry => { grow: true },
                self.button
            };
            let mut root_panel = layout! {
                StackPanel::new(Orient::Vertical),
                header_panel,
                self.canvas => { grow: true },
            };
            root_panel.set_size(csize)?;
        }

        let mut ctx = self.canvas.context()?;
        let is_dark = ColorTheme::current()? == ColorTheme::Dark;
        let brush = SolidColorBrush::new(if is_dark {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(0, 0, 0, 255)
        });
        ctx.draw_str(
            &brush,
            DrawingFontBuilder::new()
                .halign(HAlign::Left)
                .valign(VAlign::Top)
                .family("Courier New")
                .size(12.0)
                .build(),
            Point::zero(),
            match &self.text {
                NetFetchStatus::Loading => "Loading...",
                NetFetchStatus::Complete(s) => s.as_str(),
                NetFetchStatus::Error(e) => e.as_str(),
                NetFetchStatus::Timedout => "Timed out.",
            },
        )?;
        Ok(())
    }
}

impl Deref for NetPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

async fn fetch(client: Client, url: String, sender: ComponentSender<NetPage>) {
    sender.post(NetPageMessage::Fetch(NetFetchStatus::Loading));
    let status =
        if let Ok(res) = timeout(Duration::from_secs(8), client.get(url).unwrap().send()).await {
            match res {
                Ok(response) => NetFetchStatus::Complete(response.text().await.unwrap()),
                Err(e) => NetFetchStatus::Error(format!("{e:?}")),
            }
        } else {
            NetFetchStatus::Timedout
        };
    sender.post(NetPageMessage::Fetch(status));
}
