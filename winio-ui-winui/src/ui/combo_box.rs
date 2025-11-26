use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    Foundation::{IReference, TypedEventHandler},
    core::{HSTRING, IInspectable, Interface},
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::Controls::{self as MUXC, SelectionChangedEventHandler};

use crate::{GlobalRuntime, Result, Widget, ui::ToIReference};

#[derive(Debug)]
pub struct ComboBox {
    on_select: SendWrapper<Rc<Callback<()>>>,
    on_edit: SendWrapper<Rc<Callback<()>>>,
    handle: Widget,
    combo_box: MUXC::ComboBox,
}

#[inherit_methods(from = "self.handle")]
impl ComboBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let combo_box = MUXC::ComboBox::new()?;
        let on_select = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_select = on_select.clone();
            combo_box.SelectionChanged(&SelectionChangedEventHandler::new(move |_, _| {
                on_select.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }
        let on_edit = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_edit = on_edit.clone();
            combo_box.TextSubmitted(&TypedEventHandler::new(move |_, _| {
                on_edit.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }
        Ok(Self {
            on_select,
            on_edit,
            handle: Widget::new(parent, combo_box.cast()?)?,
            combo_box,
        })
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn text(&self) -> String {
        self.combo_box.Text()?.to_string_lossy()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.combo_box.SetText(&HSTRING::from(s.as_ref()))?;
    }

    pub fn selection(&self) -> Option<usize> {
        let i = self.combo_box.SelectedIndex()?;
        if i < 0 { None } else { Some(i as usize) }
    }

    pub fn set_selection(&mut self, i: usize) {
        self.combo_box.SetSelectedIndex(i as _)?;
    }

    pub fn is_editable(&self) -> bool {
        self.combo_box.IsEditable()?
    }

    pub fn set_editable(&self, v: bool) {
        self.combo_box.SetIsEditable(v)?;
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await;
    }

    pub async fn wait_change(&self) {
        self.on_edit.wait().await;
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        let item = MUXC::ComboBoxItem::new()?;
        item.SetContent(&HSTRING::from(s.as_ref()).to_reference())?;
        self.combo_box
            .Items()?
            .InsertAt(i as _, &item.cast::<IInspectable>()?)?;
        if (!self.is_editable()) && self.len() == 1 {
            self.set_selection(0);
        }
    }

    pub fn remove(&mut self, i: usize) {
        let remove_current = self.selection() == Some(i);
        self.combo_box.Items()?.RemoveAt(i as _)?;
        let len = self.len();
        if remove_current && (!self.is_editable()) {
            if len > 0 {
                self.set_selection(i.min(len - 1));
            } else {
                self.combo_box.SetSelectedIndex(-1)?;
            }
        }
    }

    pub fn get(&self, i: usize) -> String {
        let item = self.combo_box.Items()?.GetAt(i as _)?;
        item.cast::<MUXC::ComboBoxItem>()?
            .Content()?
            .cast::<IReference<HSTRING>>()?
            .Value()?
            .to_string_lossy()
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        let item = self.combo_box.Items()?.GetAt(i as _)?;
        item.cast::<MUXC::ComboBoxItem>()?
            .SetContent(&HSTRING::from(s.as_ref()).to_reference())?;
    }

    pub fn len(&self) -> usize {
        self.combo_box.Items()?.Size()? as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.combo_box.Items()?.Clear()?;
    }
}

winio_handle::impl_as_widget!(ComboBox, handle);
