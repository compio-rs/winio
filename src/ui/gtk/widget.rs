use std::rc::{Rc, Weak};

use atomic::{Atomic, Ordering};
use gtk4::prelude::{FixedExt, WidgetExt};

use crate::{Point, Size};

pub enum Container {
    Fixed(gtk4::Fixed),
    Parent(Weak<Widget>),
}

impl Container {
    pub fn add_widget(&self, widget: &gtk4::Widget) {
        match self {
            Self::Fixed(fixed) => fixed.put(widget, 0.0, 0.0),
            Self::Parent(this) => {
                if let Some(this) = this.upgrade() {
                    this.parent.add_widget(widget);
                }
            }
        }
    }

    pub fn move_widget(&self, widget: &gtk4::Widget, loc: Point) {
        match self {
            Self::Fixed(fixed) => fixed.move_(widget, loc.x, loc.y),
            Self::Parent(this) => {
                if let Some(this) = this.upgrade() {
                    this.parent
                        .move_widget(widget, loc + this.loc().to_vector());
                }
            }
        }
    }
}

pub trait AsContainer {
    fn as_container(&self) -> Container;
}

impl<T: AsContainer> AsContainer for &'_ T {
    fn as_container(&self) -> Container {
        (**self).as_container()
    }
}

impl<T: AsContainer> AsContainer for Rc<T> {
    fn as_container(&self) -> Container {
        (**self).as_container()
    }
}

pub struct Widget {
    parent: Container,
    widget: gtk4::Widget,
    loc: Atomic<Point>,
}

impl Widget {
    pub fn new(parent: impl AsContainer, widget: gtk4::Widget) -> Rc<Self> {
        let parent = parent.as_container();
        parent.add_widget(&widget);
        Rc::new(Self {
            parent,
            widget,
            loc: Atomic::new(Point::zero()),
        })
    }

    pub fn loc(&self) -> Point {
        self.loc.load(Ordering::Acquire)
    }

    pub fn set_loc(&self, p: Point) {
        self.loc.store(p, Ordering::Release);
        self.parent.move_widget(&self.widget, self.loc());
    }

    pub fn size(&self) -> Size {
        let (width, height) = self.widget.size_request();
        Size::new(width as _, height as _)
    }

    pub fn set_size(&self, s: Size) {
        self.widget.set_size_request(s.width as _, s.height as _)
    }
}

impl AsContainer for Rc<Widget> {
    fn as_container(&self) -> Container {
        Container::Parent(Rc::downgrade(self))
    }
}
