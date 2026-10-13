//! The stand-in menu bar for platforms where winio offers no menu.

use winio::prelude::*;

use crate::{Error, Result};

pub struct ExampleMenuBar;

#[derive(Debug)]
#[allow(unused)]
pub enum ExampleMenuBarEvent {
    Push,
    Pop,
    Show,
    ShowPassword(bool),
    OpenFile,
    WgpuOption(bool),
}

#[derive(Debug)]
pub enum ExampleMenuBarMessage {}

impl Component for ExampleMenuBar {
    type Error = Error;
    type Event = ExampleMenuBarEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ExampleMenuBarMessage;

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
