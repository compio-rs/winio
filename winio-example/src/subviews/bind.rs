use std::ops::Deref;

use winio::prelude::*;

use crate::{Error, Result};

pub struct BindPage {
    window: Child<TabViewItem>,
    edit1: Child<Edit>,
    edit2: Child<Edit>,
    label: Child<Label>,
    chk_1_2: Child<CheckBox>,
    chk_2_1: Child<CheckBox>,
    chk_1_label: Child<CheckBox>,
    bind_1_2: Option<usize>,
    bind_2_1: Option<usize>,
    bind_1_label: Option<usize>,
}

#[derive(Debug)]
pub enum BindPageEvent {}

pub enum BindPageMessage {
    Noop,
    Bind1To2,
    Bind2To1,
    Bind1ToLabel,
}

impl Component for BindPage {
    type Error = Error;
    type Event = BindPageEvent;
    type Init<'a> = ();
    type Message = BindPageMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: TabViewItem = (()) => {
                text: "Bind",
            },
            edit1: Edit = (&window),
            edit2: Edit = (&window),
            label: Label = (&window),
            chk_1_2: CheckBox = (&window) => {
                text: "Bind"
            },
            chk_2_1: CheckBox = (&window) => {
                text: "Bind back"
            },
            chk_1_label: CheckBox = (&window) => {
                text: "Bind"
            },
        }

        Ok(Self {
            window,
            edit1,
            edit2,
            label,
            chk_1_2,
            chk_2_1,
            chk_1_label,
            bind_1_2: None,
            bind_2_1: None,
            bind_1_label: None,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: BindPageMessage::Noop,
            self.edit1 => {},
            self.edit2 => {},
            self.label => {},
            self.chk_1_2 => {
                CheckBoxEvent::Click => BindPageMessage::Bind1To2,
            },
            self.chk_2_1 => {
                CheckBoxEvent::Click => BindPageMessage::Bind2To1,
            },
            self.chk_1_label => {
                CheckBoxEvent::Click => BindPageMessage::Bind1ToLabel,
            }
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(
            self.edit1,
            self.edit2,
            self.label,
            self.chk_1_2,
            self.chk_2_1,
            self.chk_1_label,
        )
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            BindPageMessage::Noop => {}
            BindPageMessage::Bind1To2 => {
                if self.chk_1_2.is_checked()? && self.bind_1_2.is_none() {
                    let id = self.edit1.text_prop().bind_sink(self.edit2.text_prop());
                    self.bind_1_2 = Some(id);
                } else if let Some(id) = self.bind_1_2.take() {
                    self.edit1.text_prop().unbind(id);
                }
            }
            BindPageMessage::Bind2To1 => {
                if self.chk_2_1.is_checked()? && self.bind_2_1.is_none() {
                    let id = self.edit2.text_prop().bind_sink(self.edit1.text_prop());
                    self.bind_2_1 = Some(id);
                } else if let Some(id) = self.bind_2_1.take() {
                    self.edit2.text_prop().unbind(id);
                }
            }
            BindPageMessage::Bind1ToLabel => {
                if self.chk_1_label.is_checked()? && self.bind_1_label.is_none() {
                    let id = self.edit1.text_prop().bind_sink(self.label.text_prop());
                    self.bind_1_label = Some(id);
                } else if let Some(id) = self.bind_1_label.take() {
                    self.edit1.text_prop().unbind(id);
                }
            }
        }
        Ok(false)
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.size()?;
        let margin = Margin::new_all_same(4.0);
        let mut grid = layout! {
            Grid::from_str("1*,1*,auto,auto,1*", "1*,auto,auto,auto,1*").unwrap(),
            self.edit1 => { margin: margin, column: 1, row: 1 },
            self.edit2 => { margin: margin, column: 1, row: 2 },
            self.label => { margin: margin, column: 1, row: 3 },
            self.chk_1_2 => { margin: margin, column: 2, row: 2 },
            self.chk_2_1 => { margin: margin, column: 3, row: 2 },
            self.chk_1_label => { margin: margin, column: 2, row: 3 },
        };
        grid.set_size(csize)?;
        Ok(())
    }
}

impl Deref for BindPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
