use std::{cell::RefCell, rc::Rc};

use compio_log::error;
use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows_core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, HorizontalAlignment, VerticalAlignment};

use crate::{GlobalRuntime, Result, Widget, widgets::Convertible};

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
        view.SelectionChanged({
            let on_select = on_select.clone();
            move |_, _| {
                on_select.signal::<GlobalRuntime>(());
                Ok(())
            }
        })?
        .forget();
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
        // Measure here to fix a weird issue where the TabView doesn't update
        // its measurement after selection changes.
        if let Err(_e) = self
            .view
            .Measure(Size::new(f64::INFINITY, f64::INFINITY).to_native())
        {
            error!("Measure: {_e:?}");
        }
    }
}

winio_handle::impl_as_widget!(TabView, handle);

#[derive(Debug)]
struct TabViewItemInner {
    parent: RefCell<Option<MUXC::TabView>>,
    item: MUXC::TabViewItem,
    canvas: MUXC::Canvas,
    text: MUXC::TextBlock,
}

#[derive(Debug, Clone)]
pub struct TabViewItem {
    inner: Rc<TabViewItemInner>,
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
        let item = Self {
            inner: Rc::new(TabViewItemInner {
                parent: RefCell::new(Some(parent.view.clone())),
                item,
                canvas,
                text,
            }),
        };
        let items = parent.view.TabItems()?;
        items.InsertAt(items.Size()? as _, &item.inner.item)?;
        parent
            .view
            .Measure(Size::new(f64::INFINITY, f64::INFINITY).to_native())?;
        if items.Size()? == 1 && parent.selection()?.is_none() {
            parent.view.SetSelectedIndex(0)?;
        }
        Ok(item)
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.inner.text.Text()?.to_string_lossy())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.inner.text.SetText(&HSTRING::from(s.as_ref()))?;
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        if let Some(parent) = self.inner.parent.borrow().as_ref() {
            parent.UpdateLayout()?;
            Ok(Size::new(
                self.inner.canvas.ActualWidth()?,
                self.inner.canvas.ActualHeight()?,
            ))
        } else {
            Ok(Size::zero())
        }
    }
}

impl Drop for TabViewItem {
    fn drop(&mut self) {
        let parent = self.inner.parent.borrow_mut().take();
        let Some(parent) = parent else {
            return;
        };
        let Ok(items) = parent.TabItems() else {
            return;
        };
        let mut index = 0;
        if let Ok(true) = items.IndexOf(&self.inner.item, &mut index) {
            let _ = items.RemoveAt(index);
        }
    }
}

impl AsContainer for TabViewItem {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::winui(&self.inner.canvas)
    }
}
