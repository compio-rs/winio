use inherit_methods_macro::inherit_methods;
use winio_elm::{
    Child, Component, ComponentSender, Prop, PropSink, PropSinkEvent, PropSinkMessage, start,
};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Rect, Size, TextWidget, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple radio box. See [`RadioButtonGroup`] for making selection groups.
#[derive(Debug)]
pub struct RadioButton {
    widget: sys::RadioButton,
    checked_prop: Child<Prop<bool>>,
    text_prop: Child<PropSink<String>>,
    enabled_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    tooltip_prop: Child<PropSink<String>>,
    rect_prop: Child<PropSink<Rect>>,
}

impl Failable for RadioButton {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ToolTip for RadioButton {
    fn tooltip(&self) -> Result<String>;

    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.tooltip_prop.set(s.as_ref().to_owned());
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for RadioButton {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.text_prop.set(s.as_ref().to_owned());
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl RadioButton {
    /// If the box is checked.
    pub fn is_checked(&self) -> Result<bool>;

    /// Set the checked state.
    pub fn set_checked(&mut self, v: bool) -> Result<()> {
        self.checked_prop.set(v);
        Ok(())
    }

    /// Property for [`RadioButton::is_checked`].
    pub fn checked_prop(&mut self) -> &mut Prop<bool> {
        &mut self.checked_prop
    }

    /// Property for [`TextWidget::text`].
    pub fn text_prop(&self) -> &PropSink<String> {
        &self.text_prop
    }

    /// Property for [`Enable::set_enabled`].
    pub fn enabled_prop(&self) -> &PropSink<bool> {
        &self.enabled_prop
    }

    /// Property for [`Visible::set_visible`].
    pub fn visible_prop(&self) -> &PropSink<bool> {
        &self.visible_prop
    }

    /// Property for [`ToolTip::set_tooltip`].
    pub fn tooltip_prop(&self) -> &PropSink<String> {
        &self.tooltip_prop
    }

    /// Property for [`Layoutable::rect`].
    pub fn rect_prop(&self) -> &PropSink<Rect> {
        &self.rect_prop
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for RadioButton {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()> {
        self.visible_prop.set(v);
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Enable for RadioButton {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()> {
        self.enabled_prop.set(v);
        Ok(())
    }
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for RadioButton {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()> {
        let rect = *self.rect_prop.get();
        self.rect_prop.set(Rect::new(p, rect.size));
        Ok(())
    }

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, s: Size) -> Result<()> {
        let rect = *self.rect_prop.get();
        self.rect_prop.set(Rect::new(rect.origin, s));
        Ok(())
    }

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
    /// The checked prop has been changed.
    ChangePropChecked,
    /// The checked state is set externally (e.g. from [`RadioButtonGroup`]).
    SetChecked(bool),
    /// The text has been changed.
    ChangeText,
    /// The enabled state has been changed.
    ChangeEnabled,
    /// The visible state has been changed.
    ChangeVisible,
    /// The tooltip has been changed.
    ChangeTooltip,
    /// The rect has been changed.
    ChangeRect,
}

impl Component for RadioButton {
    type Error = Error;
    type Event = RadioButtonEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = RadioButtonMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::RadioButton::new(init)?;
        let Ok(checked_prop) = Child::<Prop<bool>>::init(false).await;
        let Ok(text_prop) = Child::<PropSink<String>>::init(String::new()).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(tooltip_prop) = Child::<PropSink<String>>::init(String::new()).await;
        let loc = widget.loc()?;
        let size = widget.size()?;
        let rect = Rect::new(loc, size);
        let Ok(rect_prop) = Child::<PropSink<Rect>>::init(rect).await;
        Ok(Self {
            widget,
            checked_prop,
            text_prop,
            enabled_prop,
            visible_prop,
            tooltip_prop,
            rect_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_click = async {
            loop {
                self.widget.wait_click().await;
                sender.post(RadioButtonMessage::ChangeInputChecked);
            }
        };
        let fut_props = async {
            start! {
                sender, default: RadioButtonMessage::Noop,
                self.checked_prop => { PropSinkEvent::Changed => RadioButtonMessage::ChangePropChecked },
                self.text_prop => { PropSinkEvent::Changed => RadioButtonMessage::ChangeText },
                self.enabled_prop => { PropSinkEvent::Changed => RadioButtonMessage::ChangeEnabled },
                self.visible_prop => { PropSinkEvent::Changed => RadioButtonMessage::ChangeVisible },
                self.tooltip_prop => { PropSinkEvent::Changed => RadioButtonMessage::ChangeTooltip },
                self.rect_prop => { PropSinkEvent::Changed => RadioButtonMessage::ChangeRect },
            }
        };
        futures_util::future::join(fut_click, fut_props).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.checked_prop.update().await;
        let Ok(r1) = self.text_prop.update().await;
        let Ok(r2) = self.enabled_prop.update().await;
        let Ok(r3) = self.visible_prop.update().await;
        let Ok(r4) = self.tooltip_prop.update().await;
        let Ok(r5) = self.rect_prop.update().await;
        Ok(r0 || r1 || r2 || r3 || r4 || r5)
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
                self.checked_prop.post(PropSinkMessage::Set(checked));
                sender.output(RadioButtonEvent::Click);
                Ok(false)
            }
            RadioButtonMessage::ChangePropChecked => {
                let current = self.widget.is_checked()?;
                let prop_val = self.checked_prop.get();
                if current != *prop_val {
                    self.widget.set_checked(*prop_val)?;
                }
                Ok(true)
            }
            RadioButtonMessage::SetChecked(v) => {
                self.widget.set_checked(v)?;
                self.checked_prop.post(PropSinkMessage::Set(v));
                Ok(false)
            }
            RadioButtonMessage::ChangeText => {
                self.widget.set_text(self.text_prop.get())?;
                Ok(true)
            }
            RadioButtonMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
            RadioButtonMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            RadioButtonMessage::ChangeTooltip => {
                self.widget.set_tooltip(self.tooltip_prop.get())?;
                Ok(true)
            }
            RadioButtonMessage::ChangeRect => {
                let rect = *self.rect_prop.get();
                self.widget.set_loc(rect.origin)?;
                self.widget.set_size(rect.size)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(RadioButton, widget);

/// A group of [`RadioButton`]. Only one of them could be checked.
pub struct RadioButtonGroup {
    radios: Vec<Child<RadioButton>>,
    selection_prop: Child<Prop<Option<usize>>>,
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
    /// The selection prop has been changed.
    ChangePropSelection,
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
            self.selection_prop.post(PropSinkMessage::Set(None));
        }
    }
}

impl Component for RadioButtonGroup {
    type Error = Error;
    type Event = RadioButtonGroupEvent;
    type Init<'a> = Vec<Child<RadioButton>>;
    type Message = RadioButtonGroupMessage;

    async fn init(radios: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let Ok(selection_prop) = Child::<Prop<Option<usize>>>::init(None).await;
        Ok(Self {
            radios,
            selection_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_props = async {
            start! {
                sender, default: RadioButtonGroupMessage::Noop,
                self.selection_prop => {
                    PropSinkEvent::Changed => RadioButtonGroupMessage::ChangePropSelection,
                }
            }
        };
        let fut_radios = async {
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
            std::future::pending::<()>().await
        };
        futures_util::future::join(fut_props, fut_radios).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.selection_prop.update().await;
        let r1 = futures_util::future::try_join_all(self.radios.iter_mut().map(|c| c.update()))
            .await
            .map(|v| v.into_iter().any(|b| b))?;
        Ok(r0 || r1)
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
                self.radios[i].set_checked(true)?;
                Ok(false)
            }
            RadioButtonGroupMessage::ChangePropSelection => {
                let selection = *self.selection_prop.get();
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
