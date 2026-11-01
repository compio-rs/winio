use std::fmt::Debug;

use compio_log::error;
use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_app_kit::{NSDragOperation, NSDraggingDestination, NSDraggingInfo, NSView};
use objc2_foundation::{MainThreadMarker, NSArray, NSObjectProtocol};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, Result, Widget, catch,
    platform::{Data, drag_types},
};

/// A container that receives dragged data.
#[derive(Debug)]
pub struct DropArea {
    handle: Widget,
    view: Retained<DropView>,
}

#[inherit_methods(from = "self.handle")]
impl DropArea {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_app_kit().mtm();

        catch(|| {
            let view = DropView::new(mtm);
            view.registerForDraggedTypes(&NSArray::from_slice(&drag_types()));
            let handle = Widget::from_nsview(parent, view.clone().into_super())?;

            Ok(Self { handle, view })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub async fn wait_drop(&self) -> Data {
        self.view.ivars().on_drop.wait().await
    }
}

winio_handle::impl_as_widget!(DropArea, handle);
winio_handle::impl_as_container!(DropArea, handle);

#[derive(Debug, Default)]
struct DropViewIvars {
    on_drop: Callback<Data>,
}

define_class! {
    #[unsafe(super(NSView))]
    #[name = "WinioDropView"]
    #[ivars = DropViewIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct DropView;

    #[allow(non_snake_case)]
    impl DropView {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(DropViewIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for DropView {}

    #[allow(non_snake_case)]
    unsafe impl NSDraggingDestination for DropView {
        #[unsafe(method(draggingEntered:))]
        unsafe fn draggingEntered(
            &self,
            _sender: &ProtocolObject<dyn NSDraggingInfo>,
        ) -> NSDragOperation {
            NSDragOperation::Copy
        }

        #[unsafe(method(draggingUpdated:))]
        unsafe fn draggingUpdated(
            &self,
            _sender: &ProtocolObject<dyn NSDraggingInfo>,
        ) -> NSDragOperation {
            NSDragOperation::Copy
        }

        #[unsafe(method(prepareForDragOperation:))]
        unsafe fn prepareForDragOperation(
            &self,
            _sender: &ProtocolObject<dyn NSDraggingInfo>,
        ) -> bool {
            true
        }

        #[unsafe(method(performDragOperation:))]
        unsafe fn performDragOperation(
            &self,
            sender: &ProtocolObject<dyn NSDraggingInfo>,
        ) -> bool {
            let pasteboard = sender.draggingPasteboard();
            match Data::from_pasteboard(&pasteboard) {
                Ok(Some(data)) => {
                    self.ivars().on_drop.signal::<GlobalRuntime>(data);
                    true
                }
                Ok(None) => false,
                Err(e) => {
                    error!("Failed to read dropped data: {e:?}");
                    false
                }
            }
        }

        #[unsafe(method(wantsPeriodicDraggingUpdates))]
        unsafe fn wantsPeriodicDraggingUpdates(&self) -> bool {
            false
        }
    }
}

impl DropView {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
