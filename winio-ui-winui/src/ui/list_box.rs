use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    Foundation::IReference,
    core::{HSTRING, IInspectable, Interface},
};
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::Controls::{
    self as MUXC, SelectionChangedEventHandler, SelectionMode,
};

use crate::{GlobalRuntime, Widget, ui::ToIReference};

#[derive(Debug)]
pub struct ListBox {
    on_select: SendWrapper<Rc<Callback<()>>>,
    handle: Widget,
    list_box: MUXC::ListBox,
}

#[inherit_methods(from = "self.handle")]
impl ListBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let list_box = MUXC::ListBox::new().unwrap();
        list_box.SetSelectionMode(SelectionMode::Multiple).unwrap();
        let on_select = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_select = on_select.clone();
            list_box
                .SelectionChanged(&SelectionChangedEventHandler::new(move |_, _| {
                    on_select.signal::<GlobalRuntime>(());
                    Ok(())
                }))
                .unwrap();
        }
        Self {
            on_select,
            handle: Widget::new(parent, list_box.cast().unwrap()),
            list_box,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn min_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn is_selected(&self, i: usize) -> bool {
        self.list_box
            .Items()
            .unwrap()
            .GetAt(i as _)
            .unwrap()
            .cast::<MUXC::ListBoxItem>()
            .unwrap()
            .IsSelected()
            .unwrap()
    }

    pub fn set_selected(&mut self, i: usize, v: bool) {
        self.list_box
            .Items()
            .unwrap()
            .GetAt(i as _)
            .unwrap()
            .cast::<MUXC::ListBoxItem>()
            .unwrap()
            .SetIsSelected(v)
            .unwrap();
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await;
    }

    pub async fn wait_change(&self) {
        std::future::pending().await
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        let item = MUXC::ListBoxItem::new().unwrap();
        item.SetContent(&HSTRING::from(s.as_ref()).to_reference())
            .unwrap();
        self.list_box
            .Items()
            .unwrap()
            .InsertAt(i as _, &item.cast::<IInspectable>().unwrap())
            .unwrap();
    }

    pub fn remove(&mut self, i: usize) {
        self.list_box.Items().unwrap().RemoveAt(i as _).unwrap();
    }

    pub fn get(&self, i: usize) -> String {
        let item = self.list_box.Items().unwrap().GetAt(i as _).unwrap();
        item.cast::<MUXC::ListBoxItem>()
            .unwrap()
            .Content()
            .unwrap()
            .cast::<IReference<HSTRING>>()
            .unwrap()
            .Value()
            .unwrap()
            .to_string_lossy()
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        let item = self.list_box.Items().unwrap().GetAt(i as _).unwrap();
        item.cast::<MUXC::ListBoxItem>()
            .unwrap()
            .SetContent(&HSTRING::from(s.as_ref()).to_reference())
            .unwrap();
    }

    pub fn len(&self) -> usize {
        self.list_box.Items().unwrap().Size().unwrap() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.list_box.Items().unwrap().Clear().unwrap();
    }
}

winio_handle::impl_as_widget!(ListBox, handle);
