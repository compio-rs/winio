use objc2::rc::Id;
use objc2_app_kit::{NSTextAlignment, NSTextField};
use objc2_foundation::{MainThreadMarker, NSString};

use crate::{
    AsRawWindow, AsWindow, HAlign, Point, Size,
    ui::{Widget, from_nsstring},
};

#[derive(Debug)]
pub struct Label {
    handle: Widget,
    view: Id<NSTextField>,
}

impl Label {
    pub fn new(parent: impl AsWindow) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = NSTextField::new(mtm);
            view.setBezeled(false);
            view.setDrawsBackground(false);
            view.setEditable(false);
            view.setSelectable(false);
            let handle =
                Widget::from_nsview(parent.as_window().as_raw_window(), Id::cast(view.clone()));

            Self { handle, view }
        }
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
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

    pub fn text(&self) -> String {
        unsafe { from_nsstring(&self.view.stringValue()) }
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        unsafe {
            self.view.setStringValue(&NSString::from_str(s.as_ref()));
        }
    }

    pub fn halign(&self) -> HAlign {
        let align = unsafe { self.view.alignment() };
        match align {
            NSTextAlignment::Right => HAlign::Right,
            NSTextAlignment::Center => HAlign::Center,
            NSTextAlignment::Justified => HAlign::Stretch,
            _ => HAlign::Left,
        }
    }

    pub fn set_halign(&mut self, align: HAlign) {
        unsafe {
            let align = match align {
                HAlign::Left => NSTextAlignment::Left,
                HAlign::Center => NSTextAlignment::Center,
                HAlign::Right => NSTextAlignment::Right,
                HAlign::Stretch => NSTextAlignment::Justified,
            };
            self.view.setAlignment(align);
        }
    }
}