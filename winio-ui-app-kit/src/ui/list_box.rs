use std::cell::RefCell;

use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::{AnyObject, ProtocolObject},
};
use objc2_app_kit::{
    NSControl, NSControlTextEditingDelegate, NSFont, NSFontAttributeName, NSScrollView,
    NSStringDrawing, NSTableColumn, NSTableView, NSTableViewColumnAutoresizingStyle,
    NSTableViewDataSource, NSTableViewDelegate,
};
use objc2_foundation::{
    NSDictionary, NSIndexSet, NSInteger, NSNotification, NSObject, NSObjectProtocol, NSSize,
    NSString,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, Result, catch,
    ui::{Widget, from_cgsize},
};

#[derive(Debug)]
pub struct ListBox {
    handle: Widget,
    #[allow(unused)]
    view: Retained<NSScrollView>,
    table: Retained<NSTableView>,
    #[allow(unused)]
    column: Retained<NSTableColumn>,
    delegate: Retained<ListBoxDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl ListBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_app_kit().mtm();

        catch(|| unsafe {
            let table = NSTableView::new(mtm);
            let column = NSTableColumn::new(mtm);
            table.addTableColumn(&column);
            table.setAllowsMultipleSelection(true);
            table.setHeaderView(None);
            table.setColumnAutoresizingStyle(
                NSTableViewColumnAutoresizingStyle::UniformColumnAutoresizingStyle,
            );

            let view = NSScrollView::new(mtm);
            view.setHasVerticalScroller(true);
            view.setDocumentView(Some(&table));
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

            let delegate = ListBoxDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            table.setDelegate(Some(del_obj));
            let del_obj = ProtocolObject::from_ref(&*delegate);
            table.setDataSource(Some(del_obj));

            Ok(Self {
                handle,
                view,
                table,
                column,
                delegate,
            })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn min_size(&self) -> Result<Size> {
        catch(|| unsafe {
            let font = NSFont::systemFontOfSize(NSFont::systemFontSize());
            let attrs = NSDictionary::from_slices(&[NSFontAttributeName], &[font.as_ref()]);
            let mut width = 0.0f64;
            let mut height = 0.0f64;
            for s in self.delegate.ivars().data.borrow().iter() {
                let s = NSString::from_str(s);
                let size = s.sizeWithAttributes(Some(&attrs));
                width = width.max(size.width);
                height = height.max(size.height);
            }
            Size::new(width + 40.0, height)
        })
    }

    pub fn preferred_size(&self) -> Result<Size> {
        let mut size = catch(|| unsafe {
            from_cgsize(
                Retained::cast_unchecked::<NSControl>(self.table.clone())
                    .sizeThatFits(NSSize::ZERO),
            )
        })?;
        size.width = self.min_size()?.width;
        Ok(size)
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub async fn wait_select(&self) {
        self.delegate.ivars().select.wait().await
    }

    pub fn is_selected(&self, i: usize) -> Result<bool> {
        catch(|| self.table.isRowSelected(i as _))
    }

    pub fn set_selected(&mut self, i: usize, v: bool) -> Result<()> {
        catch(|| {
            if v {
                self.table
                    .selectRowIndexes_byExtendingSelection(&NSIndexSet::indexSetWithIndex(i), true);
            } else {
                self.table.deselectRow(i as _);
            }
        })
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.delegate.ivars().data.borrow().len())
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.delegate.ivars().data.borrow_mut().clear();
        catch(|| self.table.reloadData())
    }

    pub fn get(&self, i: usize) -> Result<String> {
        Ok(self.delegate.ivars().data.borrow()[i].clone())
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.delegate.ivars().data.borrow_mut()[i] = s.as_ref().to_string();
        catch(|| self.table.reloadData())
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.delegate
            .ivars()
            .data
            .borrow_mut()
            .insert(i, s.as_ref().to_string());
        catch(|| self.table.reloadData())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        self.delegate.ivars().data.borrow_mut().remove(i);
        catch(|| self.table.reloadData())
    }
}

winio_handle::impl_as_widget!(ListBox, handle);

#[derive(Debug, Default)]
struct ListBoxDelegateIvars {
    select: Callback,
    data: RefCell<Vec<String>>,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioListBoxDelegate"]
    #[ivars = ListBoxDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct ListBoxDelegate;

    #[allow(non_snake_case)]
    impl ListBoxDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(ListBoxDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for ListBoxDelegate {}

    unsafe impl NSControlTextEditingDelegate for ListBoxDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSTableViewDelegate for ListBoxDelegate {
        #[unsafe(method(tableViewSelectionDidChange:))]
        unsafe fn tableViewSelectionDidChange(&self, _notification: &NSNotification) {
            self.ivars().select.signal::<GlobalRuntime>(());
        }
    }

    #[allow(non_snake_case)]
    unsafe impl NSTableViewDataSource for ListBoxDelegate {
        #[unsafe(method(numberOfRowsInTableView:))]
        unsafe fn numberOfRowsInTableView(&self, _table_view: &NSTableView) -> NSInteger {
            self.ivars().data.borrow().len() as _
        }

        #[unsafe(method_id(tableView:objectValueForTableColumn:row:))]
        unsafe fn tableView_objectValueForTableColumn_row(
            &self,
            _table_view: &NSTableView,
            _table_column: Option<&NSTableColumn>,
            row: NSInteger,
        ) -> Option<Retained<AnyObject>> {
            self.ivars().data.borrow().get(row as usize).map(|s| unsafe { Retained::cast_unchecked(NSString::from_str(s)) })
        }
    }
}

impl ListBoxDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
