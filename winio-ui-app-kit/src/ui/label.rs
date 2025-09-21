use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_app_kit::{NSTextAlignment, NSTextField};
use objc2_foundation::NSString;
use winio_handle::AsContainer;
use winio_primitive::{HAlign, Point, Size};

use crate::ui::{Widget, from_nsstring};

#[derive(Debug)]
pub struct Label {
    handle: Widget,
    view: Retained<NSTextField>,
}

#[inherit_methods(from = "self.handle")]
impl Label {
    pub fn new(parent: impl AsContainer) -> Self {
        unsafe {
            let parent = parent.as_container();

            let view = NSTextField::new(parent.mtm());
            view.setBezeled(false);
            view.setDrawsBackground(false);
            view.setEditable(false);
            view.setSelectable(false);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            Self { handle, view }
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

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

winio_handle::impl_as_widget!(Label, handle);
