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
use winui3::Microsoft::UI::Xaml::Controls::{self as WUXC, SelectionChangedEventHandler};

use crate::{GlobalRuntime, Widget, ui::ToIReference};

#[derive(Debug)]
pub struct ComboBoxImpl<const E: bool> {
    on_select: SendWrapper<Rc<Callback<()>>>,
    handle: Widget,
    combo_box: WUXC::ComboBox,
}

#[inherit_methods(from = "self.handle")]
impl<const E: bool> ComboBoxImpl<E> {
    pub fn new(parent: impl AsWindow) -> Self {
        let combo_box = WUXC::ComboBox::new().unwrap();
        combo_box.SetIsEditable(E).unwrap();
        let on_select = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_select = on_select.clone();
            combo_box
                .SelectionChanged(&SelectionChangedEventHandler::new(move |_, _| {
                    on_select.signal::<GlobalRuntime>(());
                    Ok(())
                }))
                .unwrap();
        }
        Self {
            on_select,
            handle: Widget::new(parent, combo_box.cast().unwrap()),
            combo_box,
        }
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

    pub fn text(&self) -> String {
        self.combo_box.Text().unwrap().to_string_lossy()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.combo_box.SetText(&HSTRING::from(s.as_ref())).unwrap();
    }

    pub fn selection(&self) -> Option<usize> {
        let i = self.combo_box.SelectedIndex().unwrap();
        if i < 0 { None } else { Some(i as usize) }
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        let i = if let Some(i) = i { i as i32 } else { -1 };
        self.combo_box.SetSelectedIndex(i).unwrap();
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await;
    }

    pub async fn wait_change(&self) {
        std::future::pending().await
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        let item = WUXC::ComboBoxItem::new().unwrap();
        item.SetContent(&HSTRING::from(s.as_ref()).to_reference())
            .unwrap();
        self.combo_box
            .Items()
            .unwrap()
            .InsertAt(i as _, &item.cast::<IInspectable>().unwrap())
            .unwrap();
    }

    pub fn remove(&mut self, i: usize) {
        self.combo_box.Items().unwrap().RemoveAt(i as _).unwrap();
    }

    pub fn get(&self, i: usize) -> String {
        let item = self.combo_box.Items().unwrap().GetAt(i as _).unwrap();
        item.cast::<WUXC::ComboBoxItem>()
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
        let item = self.combo_box.Items().unwrap().GetAt(i as _).unwrap();
        item.cast::<WUXC::ComboBoxItem>()
            .unwrap()
            .SetContent(&HSTRING::from(s.as_ref()).to_reference())
            .unwrap();
    }

    pub fn len(&self) -> usize {
        self.combo_box.Items().unwrap().Size().unwrap() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.combo_box.Items().unwrap().Clear().unwrap();
    }
}

pub type ComboBox = ComboBoxImpl<false>;
pub type ComboEntry = ComboBoxImpl<true>;
