use std::{fmt::Debug, rc::Rc};

use gtk4::{gdk, glib, prelude::*};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Result, platform::Data, widgets::Widget};

pub struct DropArea {
    on_drop: Rc<Callback<Data>>,
    fixed: gtk4::Fixed,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl DropArea {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let fixed = gtk4::Fixed::new();
        let mut handle = Widget::new(parent, unsafe { fixed.clone().unsafe_cast() })?;
        handle.set_visible(true)?;
        let on_drop = Rc::new(Callback::new());
        let target = gtk4::DropTargetAsync::new(None, gdk::DragAction::COPY);
        target.set_propagation_phase(gtk4::PropagationPhase::Capture);
        target.connect_accept(|_, _| true);
        target.connect_drag_enter(|_, _, _, _| gdk::DragAction::COPY);
        target.connect_drag_motion(|_, _, _, _| gdk::DragAction::COPY);
        target.connect_drop({
            let on_drop = on_drop.clone();
            move |_, drop, _, _| {
                let drop = drop.clone();
                let on_drop = on_drop.clone();
                glib::spawn_future_local(async move {
                    if let Ok(Some(data)) = Data::from_drop(&drop).await {
                        on_drop.signal::<GlobalRuntime>(data);
                    }
                    drop.finish(gdk::DragAction::COPY);
                });
                true
            }
        });
        fixed.add_controller(target);
        Ok(Self {
            on_drop,
            fixed,
            handle,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub async fn wait_drop(&self) -> Data {
        self.on_drop.wait().await
    }
}

impl Debug for DropArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropArea").finish_non_exhaustive()
    }
}

winio_handle::impl_as_widget!(DropArea, handle);

impl AsContainer for DropArea {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::gtk(&self.fixed)
    }
}
