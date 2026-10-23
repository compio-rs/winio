use std::ops::Deref;

use inherit_methods_macro::inherit_methods;
use winio::prelude::{Error as SysError, Result as SysResult, *};

use crate::{Error, Result};

pub struct BindPage {
    window: Child<GroupBox>,
    view: Child<ScrollView>,
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
    Bind1To2,
    Bind2To1,
    Bind1ToLabel,
}

impl Component for BindPage {
    type Error = Error;
    type Event = BindPageEvent;
    type Init<'a> = &'a TabViewItem;
    type Message = BindPageMessage;

    async fn init(init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: GroupBox = (init) => {
                text: "Bind"
            },
            view: ScrollView = (&window) => {
                vscroll: true,
                hscroll: true,
            },
            label_edit1: Label = (&view) => {
                text: "Edit 1:",
                halign: HAlign::Right,
            },
            label_edit2: Label = (&view) => {
                text: "Edit 2:",
                halign: HAlign::Right,
            },
            label_label: Label = (&view) => {
                text: "Label:",
                halign: HAlign::Right,
            },
            edit1: Edit = (&view),
            edit2: Edit = (&view),
            label: Label = (&view),
            chk_1_2: CheckBox = (&view) => {
                text: "Bind"
            },
            chk_2_1: CheckBox = (&view) => {
                text: "Bind back"
            },
            chk_1_label: CheckBox = (&view) => {
                text: "Bind label"
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
            view,
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
            sender,
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
        let csize = self.window.client_size()?;
        let margin = Margin::new_all_same(4.0);
        let mut grid = if csize.width >= csize.height * 2.0 {
            layout! {
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
            }
        } else {
            layout! {
                Grid::from_str("1*,auto,1*,1*", "1*,auto,auto,auto,auto,auto,auto,1*").unwrap(),
                self.label_edit1 => { margin: margin, column: 1, row: 1, valign: VAlign::Center },
                self.label_edit2 => { margin: margin, column: 1, row: 2, valign: VAlign::Center },
                self.label_label => { margin: margin, column: 1, row: 3, valign: VAlign::Center },
                self.edit1 => { margin: margin, column: 2, row: 1 },
                self.edit2 => { margin: margin, column: 2, row: 2 },
                self.label => { margin: margin, column: 2, row: 3 },
                self.chk_1_2 => { margin: margin, column: 1, column_span: 2, row: 4 },
                self.chk_2_1 => { margin: margin, column: 1, column_span: 2, row: 5 },
                self.chk_1_label => { margin: margin, column: 1, column_span: 2, row: 6 },
            }
        };
        grid.set_size(csize)?;
        self.view.set_size(csize)?;
        Ok(())
    }
}

impl Deref for BindPage {
    type Target = GroupBox;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl Failable for BindPage {
    type Error = SysError;
}

#[inherit_methods(from = "self.window")]
impl Layoutable for BindPage {
    fn loc(&self) -> SysResult<Point>;

    fn set_loc(&mut self, p: Point) -> SysResult<()>;

    fn size(&self) -> SysResult<Size>;

    fn set_size(&mut self, s: Size) -> SysResult<()>;

    fn preferred_size(&self) -> SysResult<Size>;

    fn min_size(&self) -> SysResult<Size>;
}
