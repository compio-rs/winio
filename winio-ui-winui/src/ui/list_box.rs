use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    Foundation::IReference,
    core::{HSTRING, IInspectable, Interface},
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::Controls::{
    self as MUXC, SelectionChangedEventHandler, SelectionMode,
};

use crate::{GlobalRuntime, Result, Widget, ui::ToIReference};

#[derive(Debug)]
pub struct ListBox {
    on_select: SendWrapper<Rc<Callback<()>>>,
    handle: Widget,
    list_box: MUXC::ListBox,
}

#[inherit_methods(from = "self.handle")]
impl ListBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let list_box = MUXC::ListBox::new()?;
        let on_select = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_select = on_select.clone();
            list_box.SelectionChanged(&SelectionChangedEventHandler::new(move |_, _| {
                on_select.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }
        Ok(Self {
            on_select,
            handle: Widget::new(parent, list_box.cast()?)?,
            list_box,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn min_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_multiple(&self) -> Result<bool> {
        Ok(self.list_box.SelectionMode()? != SelectionMode::Single)
    }

    pub fn set_multiple(&mut self, v: bool) -> Result<()> {
        self.list_box.SetSelectionMode(if v {
            SelectionMode::Multiple
        } else {
            SelectionMode::Single
        })?;
        Ok(())
    }

    pub fn is_selected(&self, i: usize) -> Result<bool> {
        self.list_box
            .Items()?
            .GetAt(i as _)?
            .cast::<MUXC::ListBoxItem>()?
            .IsSelected()
    }

    pub fn set_selected(&mut self, i: usize, v: bool) -> Result<()> {
        self.list_box
            .Items()?
            .GetAt(i as _)?
            .cast::<MUXC::ListBoxItem>()?
            .SetIsSelected(v)?;
        Ok(())
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await;
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        let item = MUXC::ListBoxItem::new()?;
        item.SetContent(&HSTRING::from(s.as_ref()).to_reference()?)?;
        self.list_box
            .Items()?
            .InsertAt(i as _, &item.cast::<IInspectable>()?)?;
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        self.list_box.Items()?.RemoveAt(i as _)?;
        Ok(())
    }

    pub fn get(&self, i: usize) -> Result<String> {
        let item = self.list_box.Items()?.GetAt(i as _)?;
        Ok(item
            .cast::<MUXC::ListBoxItem>()?
            .Content()?
            .cast::<IReference<HSTRING>>()?
            .Value()?
            .to_string_lossy())
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        let item = self.list_box.Items()?.GetAt(i as _)?;
        item.cast::<MUXC::ListBoxItem>()?
            .SetContent(&HSTRING::from(s.as_ref()).to_reference()?)?;
        Ok(())
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.list_box.Items()?.Size()? as usize)
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.list_box.Items()?.Clear()?;
        Ok(())
    }
}

winio_handle::impl_as_widget!(ListBox, handle);
