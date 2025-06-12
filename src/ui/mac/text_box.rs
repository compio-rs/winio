use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_app_kit::{NSTextAlignment, NSTextDelegate, NSTextView, NSTextViewDelegate};
use objc2_foundation::{MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString};

use crate::{
    AsWindow, HAlign, Point, Size,
    ui::{Callback, Widget, from_nsstring},
};

#[derive(Debug)]
pub struct TextBox {
    handle: Widget,
    text_view: Retained<NSTextView>,
    delegate: Retained<TextBoxDelegate>,
}

impl TextBox {
    pub fn new(parent: impl AsWindow) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = NSTextView::scrollableTextView(mtm);
            let text_view = Retained::cast_unchecked::<NSTextView>(view.documentView().unwrap());
            text_view.setRichText(false);
            text_view.setEditable(true);
            text_view.setSelectable(true);

            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            let delegate = TextBoxDelegate::new(mtm);
            let del_obj = ProtocolObject::from_retained(delegate.clone());
            text_view.setDelegate(Some(&del_obj));
            Self {
                handle,
                text_view,
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

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v);
    }

    pub fn text(&self) -> String {
        unsafe { from_nsstring(&self.text_view.string()) }
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        unsafe {
            self.text_view.setString(&NSString::from_str(s.as_ref()));
        }
    }

    pub fn halign(&self) -> HAlign {
        let align = unsafe { self.text_view.alignment() };
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
            self.text_view.setAlignment(align);
        }
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().changed.wait().await
    }
}

#[derive(Debug, Default, Clone)]
struct TextBoxDelegateIvars {
    changed: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioTextBoxDelegate"]
    #[ivars = TextBoxDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct TextBoxDelegate;

    #[allow(non_snake_case)]
    impl TextBoxDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(TextBoxDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for TextBoxDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSTextDelegate for TextBoxDelegate {
        #[unsafe(method(textDidChange:))]
        fn textDidChange(&self, _notification: &NSNotification) {
            self.ivars().changed.signal(());
        }
    }

    unsafe impl NSTextViewDelegate for TextBoxDelegate {}
}

impl TextBoxDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
