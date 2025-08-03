use inherit_methods_macro::inherit_methods;
use objc2::{
    AnyThread, DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_app_kit::{
    NSAttributedStringNSStringDrawing, NSFontAttributeName, NSTextAlignment, NSTextDelegate,
    NSTextView, NSTextViewDelegate,
};
use objc2_foundation::{
    MainThreadMarker, NSAttributedString, NSDictionary, NSNotification, NSObject, NSObjectProtocol,
    NSString,
};
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{HAlign, Point, Size};

use crate::{
    GlobalRuntime,
    ui::{Widget, from_cgsize, from_nsstring},
};

#[derive(Debug)]
pub struct TextBox {
    handle: Widget,
    text_view: Retained<NSTextView>,
    delegate: Retained<TextBoxDelegate>,
}

#[inherit_methods(from = "self.handle")]
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
            let del_obj = ProtocolObject::from_ref(&*delegate);
            text_view.setDelegate(Some(del_obj));
            Self {
                handle,
                text_view,
                delegate,
            }
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn min_size(&self) -> Size {
        unsafe {
            let text = self.text();
            let font = self.text_view.font();
            let text = NSAttributedString::initWithString_attributes(
                NSAttributedString::alloc(),
                &NSString::from_str(text.split('\n').next().unwrap_or(&text)),
                if let Some(font) = font {
                    Some(NSDictionary::from_slices(
                        &[NSFontAttributeName],
                        &[font.as_ref()],
                    ))
                } else {
                    None
                }
                .as_deref(),
            );
            from_cgsize(text.size())
        }
    }

    pub fn preferred_size(&self) -> Size {
        unsafe {
            let font = self.text_view.font();
            let text = NSAttributedString::initWithString_attributes(
                NSAttributedString::alloc(),
                &self.text_view.string(),
                if let Some(font) = font {
                    Some(NSDictionary::from_slices(
                        &[NSFontAttributeName],
                        &[font.as_ref()],
                    ))
                } else {
                    None
                }
                .as_deref(),
            );
            from_cgsize(text.size())
        }
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

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

winio_handle::impl_as_widget!(TextBox, handle);

#[derive(Debug, Default)]
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
            self.ivars().changed.signal::<GlobalRuntime>(());
        }
    }

    unsafe impl NSTextViewDelegate for TextBoxDelegate {}
}

impl TextBoxDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
