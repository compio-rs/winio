use std::cell::RefCell;

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
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime,
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

impl ListBox {
    pub fn new(parent: impl AsWindow) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();
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
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            let delegate = ListBoxDelegate::new(mtm);
            {
                let del_obj = ProtocolObject::from_retained(delegate.clone());
                table.setDelegate(Some(&del_obj));
            }
            {
                let del_obj = ProtocolObject::from_retained(delegate.clone());
                table.setDataSource(Some(&del_obj));
            }

            Self {
                handle,
                view,
                table,
                column,
                delegate,
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
    }

    pub fn min_size(&self) -> Size {
        unsafe {
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
        }
    }

    pub fn preferred_size(&self) -> Size {
        unsafe {
            let mut size = from_cgsize(
                Retained::cast_unchecked::<NSControl>(self.table.clone())
                    .sizeThatFits(NSSize::ZERO),
            );
            size.width = self.min_size().width;
            size
        }
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v)
    }

    pub async fn wait_select(&self) {
        self.delegate.ivars().select.wait().await
    }

    pub fn is_selected(&self, i: usize) -> bool {
        unsafe { self.table.isRowSelected(i as _) }
    }

    pub fn set_selected(&mut self, i: usize, v: bool) {
        unsafe {
            if v {
                self.table
                    .selectRowIndexes_byExtendingSelection(&NSIndexSet::indexSetWithIndex(i), true);
            } else {
                self.table.deselectRow(i as _);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.delegate.ivars().data.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.delegate.ivars().data.borrow_mut().clear();
        unsafe { self.table.reloadData() };
    }

    pub fn get(&self, i: usize) -> String {
        self.delegate.ivars().data.borrow()[i].clone()
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        self.delegate.ivars().data.borrow_mut()[i] = s.as_ref().to_string();
        unsafe { self.table.reloadData() };
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        self.delegate
            .ivars()
            .data
            .borrow_mut()
            .insert(i, s.as_ref().to_string());
        unsafe { self.table.reloadData() };
    }

    pub fn remove(&mut self, i: usize) {
        self.delegate.ivars().data.borrow_mut().remove(i);
        unsafe { self.table.reloadData() };
    }
}

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
            self.ivars().data.borrow().get(row as usize).map(|s| Retained::cast_unchecked(NSString::from_str(s)))
        }
    }
}

impl ListBoxDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
