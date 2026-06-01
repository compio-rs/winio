use std::ops::Deref;

use winio::prelude::*;

use crate::{Error, Result};

pub struct DummyPage {
    window: Child<TabViewItem>,
    label: Child<Label>,
}

#[derive(Debug)]
pub enum DummyPageEvent {}

#[derive(Debug)]
pub enum DummyPageMessage {
    Noop,
}

impl Component for DummyPage {
    type Error = Error;
    type Event = DummyPageEvent;
    type Init<'a> = (&'static str, &'static str);
    type Message = DummyPageMessage;

    async fn init(
        (name, feature): Self::Init<'_>,
        _sender: &ComponentSender<Self>,
    ) -> Result<Self> {
        init! {
            window: TabViewItem = (()) => {
                text: name,
            },
            label: Label = (&window) => {
                text: format!("Please enable the \"{}\" feature to see this page.", feature),
                halign: HAlign::Center,
            },
        }

        Ok(Self { window, label })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: DummyPageMessage::Noop,
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(self.window, self.label)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            DummyPageMessage::Noop => Ok(false),
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.size()?;
        {
            let mut grid = layout! {
                Grid::from_str("1*,2*,1*", "1*,2*,1*").unwrap(),
                self.label => { column: 1, row: 1 },
            };
            grid.set_size(csize)?;
        }
        Ok(())
    }
}

impl Deref for DummyPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
