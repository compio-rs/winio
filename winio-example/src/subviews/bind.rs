use std::ops::Deref;

use winio::prelude::*;

use crate::{Error, Result};

pub struct BindPage {
    window: Child<TabViewItem>,
    label_edit1: Child<Label>,
    label_edit2: Child<Label>,
    label_label: Child<Label>,
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

    async fn init(_init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: TabViewItem = (()) => {
                text: "Bind",
            },
            label_edit1: Label = (&window) => {
                text: "Edit 1:",
                halign: HAlign::Right,
            },
            label_edit2: Label = (&window) => {
                text: "Edit 2:",
                halign: HAlign::Right,
            },
            label_label: Label = (&window) => {
                text: "Label:",
                halign: HAlign::Right,
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

        chk_1_2
            .checked_prop()?
            .bind(sender, |_| BindPageMessage::Bind1To2);
        chk_2_1
            .checked_prop()?
            .bind(sender, |_| BindPageMessage::Bind2To1);
        chk_1_label
            .checked_prop()?
            .bind(sender, |_| BindPageMessage::Bind1ToLabel);

        Ok(Self {
            window,
            label_edit1,
            label_edit2,
            label_label,
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
            self.window => {},
            self.edit1 => {},
            self.edit2 => {},
            self.label => {},
            self.chk_1_2 => {},
            self.chk_2_1 => {},
            self.chk_1_label => {},
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(
            self.window,
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
                    let id = self
                        .edit1
                        .text_prop()?
                        .bind(self.edit2.sender(), EditMessage::SetText);
                    self.bind_1_2 = Some(id);
                } else if let Some(id) = self.bind_1_2.take() {
                    self.edit1.text_prop()?.unbind(id);
                }
            }
            BindPageMessage::Bind2To1 => {
                if self.chk_2_1.is_checked()? && self.bind_2_1.is_none() {
                    let id = self
                        .edit2
                        .text_prop()?
                        .bind(self.edit1.sender(), EditMessage::SetText);
                    self.bind_2_1 = Some(id);
                } else if let Some(id) = self.bind_2_1.take() {
                    self.edit2.text_prop()?.unbind(id);
                }
            }
            BindPageMessage::Bind1ToLabel => {
                if self.chk_1_label.is_checked()? && self.bind_1_label.is_none() {
                    let id = self
                        .edit1
                        .text_prop()?
                        .bind(self.label.sender(), LabelMessage::SetText);
                    self.bind_1_label = Some(id);
                } else if let Some(id) = self.bind_1_label.take() {
                    self.edit1.text_prop()?.unbind(id);
                }
            }
        }
        Ok(false)
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.size()?;
        let margin = Margin::new_all_same(4.0);
        let mut grid = layout! {
            Grid::from_str("1*,auto,1*,auto,auto,1*", "1*,auto,auto,auto,1*").unwrap(),
            self.label_edit1 => { margin: margin, column: 1, row: 1, valign: VAlign::Center },
            self.label_edit2 => { margin: margin, column: 1, row: 2, valign: VAlign::Center },
            self.label_label => { margin: margin, column: 1, row: 3, valign: VAlign::Center },
            self.edit1 => { margin: margin, column: 2, row: 1 },
            self.edit2 => { margin: margin, column: 2, row: 2 },
            self.label => { margin: margin, column: 2, row: 3 },
            self.chk_1_2 => { margin: margin, column: 3, row: 2 },
            self.chk_2_1 => { margin: margin, column: 4, row: 2 },
            self.chk_1_label => { margin: margin, column: 3, row: 3 },
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
