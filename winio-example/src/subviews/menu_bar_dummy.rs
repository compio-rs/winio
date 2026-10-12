//! The stand-in menu bar for platforms where winio offers no menu.

use winio::prelude::*;

use crate::{Error, Result};

pub struct DummyMenuBar;

#[derive(Debug)]
pub enum DummyMenuBarEvent {
    Push,
    Pop,
    Show,
    ShowPassword(bool),
    OpenFile,
    WgpuOption(bool),
}

#[derive(Debug)]
pub enum DummyMenuBarMessage {}

impl Component for DummyMenuBar {
    type Error = Error;
    type Event = DummyMenuBarEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = DummyMenuBarMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        Ok(Self)
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        std::future::pending().await
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {}
    }
}
