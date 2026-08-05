use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, Prop};
use winio_handle::BorrowedContainer;
use winio_primitive::{
    Enable, Failable, Layoutable, Point, Rect, Size, TextWidget, ToolTip, Visible,
};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple radio box. See [`RadioButtonGroup`] for making selection groups.
#[derive(Debug)]
pub struct RadioButton {
    widget: sys::RadioButton,
    checked_prop: Prop<bool>,
}

impl Failable for RadioButton {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for RadioButton {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for RadioButton {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl RadioButton {
    /// If the box is checked.
    pub fn is_checked(&self) -> Result<bool>;

    /// Set the checked state.
    pub fn set_checked(&mut self, v: bool) -> Result<()> {
        self.widget.set_checked(v)?;
        self.checked_prop.set(v);
        Ok(())
    }

    /// Property for [`RadioButton::is_checked`].
    pub fn checked_prop(&mut self) -> &mut Prop<bool> {
        &mut self.checked_prop
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for RadioButton {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for RadioButton {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for RadioButton {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`RadioButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum RadioButtonEvent {
    /// The check box has been clicked.
    Click,
}

/// Messages of [`RadioButton`].
#[derive(Debug)]
#[non_exhaustive]
pub enum RadioButtonMessage {
    /// No operation.
    Noop,
    /// The checked state has been changed by user click.
    ChangeInputChecked,
    /// The checked state is set externally (e.g. from [`RadioButtonGroup`]).
    SetChecked(bool),
    /// Set the rect.
    SetRect(Rect),
    /// Set the text.
    SetText(String),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
    /// Set the tooltip.
    SetTooltip(String),
}

impl Component for RadioButton {
    type Error = Error;
    type Event = RadioButtonEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = RadioButtonMessage;

    async fn init(init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::RadioButton::new(init)?;
        let mut checked_prop = Prop::new(false);
        checked_prop.bind(sender, RadioButtonMessage::SetChecked);
        Ok(Self {
            widget,
            checked_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_click().await;
            sender.post(RadioButtonMessage::ChangeInputChecked);
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            RadioButtonMessage::Noop => Ok(false),
            RadioButtonMessage::ChangeInputChecked => {
                let checked = self.widget.is_checked()?;
                self.checked_prop.set(checked);
                sender.output(RadioButtonEvent::Click);
                Ok(false)
            }
            RadioButtonMessage::SetChecked(v) => {
                self.set_checked(v)?;
                Ok(true)
            }
            RadioButtonMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            RadioButtonMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
            RadioButtonMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            RadioButtonMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
            RadioButtonMessage::SetTooltip(tooltip) => {
                self.set_tooltip(tooltip)?;
                Ok(false)
            }
        }
    }
}

winio_handle::impl_as_widget!(RadioButton, widget);

/// A group of [`RadioButton`]. Only one of them could be checked.
pub struct RadioButtonGroup {
    radios: Vec<Child<RadioButton>>,
    selection_prop: Prop<Option<usize>>,
}

/// Events of [`RadioButtonGroup`].
#[derive(Debug)]
#[non_exhaustive]
pub enum RadioButtonGroupEvent {}

/// Messages of [`RadioButtonGroup`].
#[derive(Debug)]
#[non_exhaustive]
pub enum RadioButtonGroupMessage {
    /// No operation.
    Noop,
    /// A radio button has been selected, with its index.
    Click(usize),
    /// Set the selection.
    SetSelection(Option<usize>),
}

impl RadioButtonGroup {
    /// Get the index of the selected radio button, or `None` if none is
    /// selected.
    pub fn selection(&self) -> Result<Option<usize>> {
        Ok(*self.selection_prop.get())
    }

    /// Set the index of the selected radio button.
    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        self.selection_prop.set(Some(i));
        Ok(())
    }

    /// Property for the selected radio button index.
    pub fn selection_prop(&mut self) -> &mut Prop<Option<usize>> {
        &mut self.selection_prop
    }

    /// Appends a radio button to the back of the group.
    pub fn push(&mut self, v: Child<RadioButton>) {
        self.radios.push(v);
        self.fix_selection();
    }

    /// Inserts a radio button at specified position.
    pub fn insert(&mut self, i: usize, v: Child<RadioButton>) {
        self.radios.insert(i, v);
        self.fix_selection();
    }

    /// Removes the last radio button.
    pub fn pop(&mut self) -> Option<Child<RadioButton>> {
        let res = self.radios.pop();
        self.fix_selection();
        res
    }

    /// Removes and returns the radio button at specified position.
    pub fn remove(&mut self, i: usize) -> Child<RadioButton> {
        let res = self.radios.remove(i);
        self.fix_selection();
        res
    }

    /// Replaces the radio button at specified position, and returns the old
    /// one.
    pub fn replace(&mut self, i: usize, v: Child<RadioButton>) -> Child<RadioButton> {
        std::mem::replace(&mut self.radios[i], v)
    }

    /// Clears the group.
    pub fn clear(&mut self) {
        self.radios.clear();
        self.fix_selection();
    }

    /// Shrinks the capacity of the group as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.radios.shrink_to_fit();
    }

    /// Length of the group.
    pub fn len(&self) -> usize {
        self.radios.len()
    }

    /// Checks if the group is empty.
    pub fn is_empty(&self) -> bool {
        self.radios.is_empty()
    }

    /// Gets the inner radio buttons.
    pub fn items(&self) -> &[Child<RadioButton>] {
        &self.radios
    }

    /// Gets the inner radio buttons mutably.
    pub fn items_mut(&mut self) -> &mut [Child<RadioButton>] {
        &mut self.radios
    }

    /// Gets the radio button at specified position.
    pub fn get(&self, i: usize) -> Option<&Child<RadioButton>> {
        self.radios.get(i)
    }

    /// Resets the selection if the index is out of bounds.
    fn fix_selection(&mut self) {
        let len = self.radios.len();
        if let Some(i) = *self.selection_prop.get()
            && i >= len
        {
            self.selection_prop.set(None);
        }
    }
}

impl Component for RadioButtonGroup {
    type Error = Error;
    type Event = RadioButtonGroupEvent;
    type Init<'a> = Vec<Child<RadioButton>>;
    type Message = RadioButtonGroupMessage;

    async fn init(radios: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        let mut selection_prop = Prop::new(None);
        selection_prop.bind(sender, RadioButtonGroupMessage::SetSelection);
        Ok(Self {
            radios,
            selection_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let futures = self
            .radios
            .iter_mut()
            .enumerate()
            .map(|(i, c)| {
                c.start(
                    sender,
                    move |e| match e {
                        RadioButtonEvent::Click => Some(RadioButtonGroupMessage::Click(i)),
                    },
                    || RadioButtonGroupMessage::Noop,
                )
            })
            .collect::<Vec<_>>();
        futures_util::future::join_all(futures).await;
        std::future::pending().await
    }

    async fn update_children(&mut self) -> Result<bool> {
        let r0 = futures_util::future::try_join_all(self.radios.iter_mut().map(|c| c.update()))
            .await
            .map(|v| v.into_iter().any(|b| b))?;
        Ok(r0)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            RadioButtonGroupMessage::Noop => Ok(false),
            RadioButtonGroupMessage::Click(i) => {
                self.selection_prop.set(Some(i));
                Ok(false)
            }
            RadioButtonGroupMessage::SetSelection(selection) => {
                self.selection_prop.set(selection);
                if let Some(i) = selection {
                    for (idx, r) in self.radios.iter_mut().enumerate() {
                        r.set_checked(idx == i)?;
                    }
                }
                Ok(false)
            }
        }
    }
}
