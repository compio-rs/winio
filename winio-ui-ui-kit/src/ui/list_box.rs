use std::cell::RefCell;

use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_core_foundation::CGRect;
use objc2_foundation::{NSIndexPath, NSInteger, NSObject, NSObjectProtocol, NSString, ns_string};
use objc2_ui_kit::{
    NSIndexPathUIKitAdditions, UIScrollViewDelegate, UITableView, UITableViewCell,
    UITableViewCellStyle, UITableViewDataSource, UITableViewDelegate, UITableViewScrollPosition,
    UITableViewStyle,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Result, catch, ui::Widget};

#[derive(Debug)]
pub struct ListBox {
    handle: Widget,
    table: Retained<UITableView>,
    delegate: Retained<ListBoxDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl ListBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| unsafe {
            let table = UITableView::initWithFrame_style(
                UITableView::alloc(mtm),
                CGRect::ZERO,
                UITableViewStyle::Plain,
            );

            let delegate = ListBoxDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            table.setDelegate(Some(del_obj));
            let del_obj = ProtocolObject::from_ref(&*delegate);
            table.setDataSource(Some(del_obj));

            let handle = Widget::from_uiview(parent, Retained::cast_unchecked(table.clone()))?;

            Ok(Self {
                handle,
                table,
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
        Ok(Size::new(50.0, 20.0))
    }

    pub fn preferred_size(&self) -> Result<Size> {
        Ok(Size::new(100.0, 100.0))
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

    pub fn is_multiple(&self) -> Result<bool> {
        catch(|| self.table.allowsMultipleSelection())
    }

    pub fn set_multiple(&mut self, v: bool) -> Result<()> {
        catch(|| self.table.setAllowsMultipleSelection(v))
    }

    pub fn is_selected(&self, i: usize) -> Result<bool> {
        catch(|| {
            if let Some(selected) = self.table.indexPathsForSelectedRows() {
                selected.iter().any(|ip| ip.row() as usize == i)
            } else {
                false
            }
        })
    }

    pub fn set_selected(&mut self, i: usize, v: bool) -> Result<()> {
        catch(|| {
            let ip = NSIndexPath::indexPathForRow_inSection(i as _, 0);
            if v {
                self.table.selectRowAtIndexPath_animated_scrollPosition(
                    Some(&ip),
                    false,
                    UITableViewScrollPosition::None,
                );
            } else {
                self.table.deselectRowAtIndexPath_animated(&ip, false);
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
    #[name = "WinioListBoxDelegateUIKit"]
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

    unsafe impl UIScrollViewDelegate for ListBoxDelegate {}

    #[allow(non_snake_case)]
    unsafe impl UITableViewDelegate for ListBoxDelegate {
        #[unsafe(method(tableView:didSelectRowAtIndexPath:))]
        unsafe fn tableView_didSelectRowAtIndexPath(
            &self,
            _table_view: &UITableView,
            _index_path: &NSIndexPath,
        ) {
            self.ivars().select.signal::<GlobalRuntime>(());
        }
    }

    #[allow(non_snake_case)]
    unsafe impl UITableViewDataSource for ListBoxDelegate {
        #[unsafe(method(tableView:numberOfRowsInSection:))]
        fn tableView_numberOfRowsInSection(
            &self,
            _table_view: &UITableView,
            _section: NSInteger,
        ) -> NSInteger {
            self.ivars().data.borrow().len() as _
        }

        #[unsafe(method_id(tableView:cellForRowAtIndexPath:))]
        fn tableView_cellForRowAtIndexPath(
            &self,
            table_view: &UITableView,
            index_path: &NSIndexPath,
        ) -> Option<Retained<UITableViewCell>> {
            let cell_id = ns_string!("Cell");
            let cell = table_view
                .dequeueReusableCellWithIdentifier(cell_id)
                .unwrap_or_else(|| {
                    let mtm = table_view.mtm();
                    UITableViewCell::initWithStyle_reuseIdentifier(
                        UITableViewCell::alloc(mtm),
                        UITableViewCellStyle::Default,
                        Some(cell_id),
                    )
                });
            let data = self.ivars().data.borrow();
            if let Some(text) = data.get(index_path.row() as usize) {
                let config = cell.defaultContentConfiguration();
                config.setText(Some(&NSString::from_str(text)));
                let config = ProtocolObject::from_ref(&*config);
                cell.setContentConfiguration(Some(config));
            }
            Some(cell)
        }
    }
}

impl ListBoxDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
