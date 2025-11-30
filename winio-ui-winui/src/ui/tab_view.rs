use std::rc::Rc;

use compio_log::error;
use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::{AsContainer, AsRawContainer, RawContainer};
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, HorizontalAlignment, VerticalAlignment};

use crate::{GlobalRuntime, Result, Widget, ui::Convertible};

#[derive(Debug)]
pub struct TabView {
    on_select: SendWrapper<Rc<Callback>>,
    handle: Widget,
    view: MUXC::TabView,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let on_select = SendWrapper::new(Rc::new(Callback::new()));
        let view = MUXC::TabView::new()?;
        view.SelectionChanged(&MUXC::SelectionChangedEventHandler::new({
            let on_select = on_select.clone();
            move |_, _| {
                on_select.signal::<GlobalRuntime>(());
                Ok(())
            }
        }))?;
        view.SetHorizontalContentAlignment(HorizontalAlignment::Stretch)?;
        view.SetVerticalContentAlignment(VerticalAlignment::Stretch)?;
        view.SetIsAddTabButtonVisible(false)?;
        Ok(Self {
            on_select,
            handle: Widget::new(parent, view.cast()?)?,
            view,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn selection(&self) -> Result<Option<usize>> {
        let i = self.view.SelectedIndex()?;
        Ok(if i < 0 { None } else { Some(i as usize) })
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        self.view.SetSelectedIndex(i as _)?;
        Ok(())
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await;
        // Measure here to fix a weird issue where the TabView doesn't update its
        // measurement after selection changes.
        if let Err(_e) = self
            .view
            .Measure(Size::new(f64::INFINITY, f64::INFINITY).to_native())
        {
            error!("Measure: {_e:?}");
        }
    }

    pub fn insert(&mut self, i: usize, item: &TabViewItem) -> Result<()> {
        self.view.TabItems()?.InsertAt(i as _, &item.item)?;
        self.view
            .Measure(Size::new(f64::INFINITY, f64::INFINITY).to_native())?;
        if self.len()? == 1 {
            self.set_selection(0)?;
        }
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        self.view.TabItems()?.RemoveAt(i as _)?;
        Ok(())
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.view.TabItems()?.Size()? as _)
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.view.TabItems()?.Clear()?;
        Ok(())
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
    pub fn new(parent: &TabView) -> Result<Self> {
        let item = MUXC::TabViewItem::new()?;
        item.SetHorizontalAlignment(HorizontalAlignment::Stretch)?;
        item.SetVerticalAlignment(VerticalAlignment::Stretch)?;
        item.SetIsClosable(false)?;
        let text = MUXC::TextBlock::new()?;
        item.SetHeader(&text)?;
        let canvas = MUXC::Canvas::new()?;
        canvas.SetHorizontalAlignment(HorizontalAlignment::Stretch)?;
        canvas.SetVerticalAlignment(VerticalAlignment::Stretch)?;
        item.SetContent(&canvas)?;
        Ok(Self {
            parent: parent.view.clone(),
            item,
            canvas,
            text,
        })
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.text.Text()?.to_string_lossy())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.text.SetText(&HSTRING::from(s.as_ref()))?;
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        Ok(Size::new(
            self.parent.Width()?,
            (self.parent.Height()? - 40.0).max(0.0),
        ))
    }
}

impl AsRawContainer for TabViewItem {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::WinUI(self.canvas.clone())
    }
}

winio_handle::impl_as_container!(TabViewItem);
