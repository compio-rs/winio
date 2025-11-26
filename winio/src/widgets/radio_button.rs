use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, TextWidget, ToolTip, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A simple radio box. See [`RadioButtonGroup`] for making selection groups.
#[derive(Debug)]
pub struct RadioButton {
    widget: sys::RadioButton,
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
    pub fn set_checked(&mut self, v: bool) -> Result<()>;
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

    fn set_size(&mut self, v: Size) -> Result<()>;

    fn preferred_size(&self) -> Result<Size>;
}

/// Events of [`RadioButton`].
#[non_exhaustive]
pub enum RadioButtonEvent {
    /// The check box has been clicked.
    Click,
}

impl Component for RadioButton {
    type Error = Error;
    type Event = RadioButtonEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::RadioButton::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_click().await;
            sender.output(RadioButtonEvent::Click);
        }
    }
}

winio_handle::impl_as_widget!(RadioButton, widget);

/// A group of [`RadioButton`]. Only one of them could be checked.
pub struct RadioButtonGroup<T> {
    radios: T,
}

impl<'a, T: AsMut<[&'a mut RadioButton]>> RadioButtonGroup<T> {
    /// Create [`RadioButtonGroup`].
    pub fn new(radios: T) -> Self {
        Self { radios }
    }

    /// Start listening the click events of the radio boxes.
    pub async fn start<C: Component>(
        &mut self,
        sender: &ComponentSender<C>,
        mut f: impl FnMut(usize) -> Option<C::Message>,
        _propagate: impl FnMut() -> C::Message,
    ) -> ! {
        if self.radios.as_mut().is_empty() {
            std::future::pending::<()>().await;
        }
        loop {
            let ((), index, _) = futures_util::future::select_all(
                self.radios
                    .as_mut()
                    .iter()
                    .map(|r| Box::pin(r.widget.wait_click())),
            )
            .await;
            for (i, r) in self.radios.as_mut().iter_mut().enumerate() {
                if i != index {
                    r.set_checked(false);
                }
            }
            if let Some(message) = f(index) {
                sender.post(message);
            }
        }
    }
}
