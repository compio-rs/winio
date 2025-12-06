use std::{cell::RefCell, rc::Rc};

use compio_log::error;
use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::core::{HSTRING, Interface};
use winio_callback::Callback;
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, HorizontalAlignment, VerticalAlignment};

use crate::{GlobalRuntime, Result, Widget, ui::Convertible};

#[derive(Debug)]
pub struct TabView {
    on_select: SendWrapper<Rc<Callback>>,
    handle: Widget,
    view: MUXC::TabView,
    views: Vec<TabViewItem>,
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
            views: vec![],
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
        self.view.TabItems()?.InsertAt(i as _, &item.inner.item)?;
        item.inner.parent.replace(Some(self.view.clone()));
        self.views.insert(i, item.clone());
        self.view
            .Measure(Size::new(f64::INFINITY, f64::INFINITY).to_native())?;
        if self.len()? == 1 && self.selection()?.is_none() {
            self.set_selection(0)?;
        }
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        self.view.TabItems()?.RemoveAt(i as _)?;
        self.views.remove(i).inner.parent.replace(None);
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
        for item in self.views.drain(..) {
            item.inner.parent.replace(None);
        }
        Ok(())
    }
}

winio_handle::impl_as_widget!(TabView, handle);

#[derive(Debug)]
struct TabViewInner {
    parent: RefCell<Option<MUXC::TabView>>,
    item: MUXC::TabViewItem,
    canvas: MUXC::Canvas,
    text: MUXC::TextBlock,
}

#[derive(Debug, Clone)]
pub struct TabViewItem {
    inner: Rc<TabViewInner>,
}

impl TabViewItem {
    pub fn new() -> Result<Self> {
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
            inner: Rc::new(TabViewInner {
                parent: RefCell::new(None),
                item,
                canvas,
                text,
            }),
        })
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
            Ok(Size::new(
                parent.Width()?,
                (parent.Height()? - 40.0).max(0.0),
            ))
        } else {
            Ok(Size::zero())
        }
    }
}

impl AsContainer for TabViewItem {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::winui(&self.inner.canvas)
    }
}
