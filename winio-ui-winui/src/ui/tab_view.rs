use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::{AsContainer, AsRawContainer, RawContainer};
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, HorizontalAlignment, VerticalAlignment};

use crate::{GlobalRuntime, Widget, ui::Convertible};

#[derive(Debug)]
pub struct TabView {
    on_select: SendWrapper<Rc<Callback>>,
    handle: Widget,
    view: MUXC::TabView,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Self {
        let on_select = SendWrapper::new(Rc::new(Callback::new()));
        let view = MUXC::TabView::new().unwrap();
        view.SelectionChanged(&MUXC::SelectionChangedEventHandler::new({
            let on_select = on_select.clone();
            move |_, _| {
                on_select.signal::<GlobalRuntime>(());
                Ok(())
            }
        }))
        .unwrap();
        view.SetHorizontalContentAlignment(HorizontalAlignment::Stretch)
            .unwrap();
        view.SetVerticalContentAlignment(VerticalAlignment::Stretch)
            .unwrap();
        view.SetIsAddTabButtonVisible(false).unwrap();
        Self {
            on_select,
            handle: Widget::new(parent, view.cast().unwrap()),
            view,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn selection(&self) -> Option<usize> {
        self.view
            .SelectedIndex()
            .ok()
            .and_then(|i| if i < 0 { None } else { Some(i as usize) })
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        let i = i.map(|i| i as i32).unwrap_or(-1);
        self.view.SetSelectedIndex(i).unwrap();
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await
    }

    pub fn insert(&mut self, i: usize, item: &TabViewItem) {
        self.view
            .TabItems()
            .unwrap()
            .InsertAt(i as _, &item.item)
            .unwrap();
        self.view
            .Measure(Size::new(f64::INFINITY, f64::INFINITY).to_native())
            .unwrap();
    }

    pub fn remove(&mut self, i: usize) {
        self.view.TabItems().unwrap().RemoveAt(i as _).unwrap();
    }

    pub fn len(&self) -> usize {
        self.view.TabItems().unwrap().Size().unwrap() as _
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.view.TabItems().unwrap().Clear().unwrap();
    }
}

winio_handle::impl_as_widget!(TabView, handle);

#[derive(Debug)]
pub struct TabViewItem {
    parent: MUXC::TabView,
    item: MUXC::TabViewItem,
    canvas: MUXC::Canvas,
    text: MUXC::TextBlock,
}

impl TabViewItem {
    pub fn new(parent: &TabView) -> Self {
        let item = MUXC::TabViewItem::new().unwrap();
        item.SetHorizontalAlignment(HorizontalAlignment::Stretch)
            .unwrap();
        item.SetVerticalAlignment(VerticalAlignment::Stretch)
            .unwrap();
        item.SetIsClosable(false).unwrap();
        let text = MUXC::TextBlock::new().unwrap();
        item.SetHeader(&text).unwrap();
        let canvas = MUXC::Canvas::new().unwrap();
        canvas
            .SetHorizontalAlignment(HorizontalAlignment::Stretch)
            .unwrap();
        canvas
            .SetVerticalAlignment(VerticalAlignment::Stretch)
            .unwrap();
        item.SetContent(&canvas).unwrap();
        Self {
            parent: parent.view.clone(),
            item,
            canvas,
            text,
        }
    }

    pub fn text(&self) -> String {
        self.text.Text().unwrap().to_string_lossy()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.text.SetText(&HSTRING::from(s.as_ref())).unwrap();
    }

    pub fn size(&self) -> Size {
        Size::new(
            self.parent.Width().unwrap(),
            (self.parent.Height().unwrap() - 40.0).max(0.0),
        )
    }
}

impl AsRawContainer for TabViewItem {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::WinUI(self.canvas.clone())
    }
}

winio_handle::impl_as_container!(TabViewItem);
