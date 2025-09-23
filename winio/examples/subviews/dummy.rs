use std::ops::Deref;

use tuplex::IntoArray;
use winio::prelude::*;

pub struct DummyPage {
    window: Child<TabViewItem>,
    label: Child<Label>,
}

#[derive(Debug)]
pub enum DummyPageMessage {
    Noop,
}

impl Component for DummyPage {
    type Event = ();
    type Init<'a> = (&'a TabView, &'static str, &'static str);
    type Message = DummyPageMessage;

    fn init((tabview, name, feature): Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        init! {
            window: TabViewItem = (tabview) => {
                text: name,
            },
            label: Label = (&window) => {
                text: format!("Please enable the \"{}\" feature to see this page.", feature),
                halign: HAlign::Center,
            },
        }

        Self { window, label }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: DummyPageMessage::Noop,
        }
    }

    async fn update_children(&mut self) -> bool {
        futures_util::future::join(self.window.update(), self.label.update())
            .await
            .into_array()
            .into_iter()
            .any(|b| b)
    }

    async fn update(&mut self, message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        match message {
            DummyPageMessage::Noop => false,
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.size();
        {
            let mut grid = layout! {
                Grid::from_str("1*,2*,1*", "1*,2*,1*").unwrap(),
                self.label => { column: 1, row: 1 },
            };
            grid.set_size(csize);
        }
    }
}

impl Deref for DummyPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
