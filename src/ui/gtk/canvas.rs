use std::{
    io,
    rc::{Rc, Weak},
};

use glib::object::Cast;
use gtk4::{
    cairo::Context,
    prelude::{DrawingAreaExtManual, WidgetExt},
    DrawingArea,
};

use super::callback::Callback;
use crate::{AsContainer, Container, Point, Size, Widget};

pub struct Canvas {
    widget: DrawingArea,
    handle: Rc<Widget>,
    on_redraw: Callback<Context>,
}

impl Canvas {
    pub fn new(parent: impl AsContainer) -> io::Result<Rc<Self>> {
        let widget = DrawingArea::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        Ok(Rc::new_cyclic(|this: &Weak<Canvas>| {
            widget.set_draw_func({
                let this = this.clone();
                move |_, ctx, _, _| {
                    if let Some(this) = this.upgrade() {
                        this.on_redraw.signal(ctx.clone());
                    }
                }
            });
            Self {
                widget,
                handle,
                on_redraw: Callback::new(),
            }
        }))
    }

    pub fn loc(&self) -> io::Result<Point> {
        Ok(self.handle.loc())
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        self.handle.set_loc(p);
        Ok(())
    }

    pub fn size(&self) -> io::Result<Size> {
        Ok(self.handle.size())
    }

    pub fn set_size(&self, s: Size) -> io::Result<()> {
        self.handle.set_size(s);
        Ok(())
    }

    pub fn redraw(&self) -> io::Result<()> {
        self.widget.queue_draw();
        Ok(())
    }

    pub async fn wait_redraw(&self) -> io::Result<DrawingContext> {
        let ctx = self.on_redraw.wait().await;
        Ok(DrawingContext { ctx })
    }
}

impl AsContainer for Canvas {
    fn as_container(&self) -> Container {
        Container::Parent(Rc::downgrade(&self.handle))
    }
}

pub struct DrawingContext {
    ctx: Context,
}

pub trait Brush {}

pub trait Pen {}
