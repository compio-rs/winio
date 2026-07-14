use inherit_methods_macro::inherit_methods;
use objc2::{
    AnyThread, DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_foundation::{
    MainThreadMarker, NSAttributedString, NSDictionary, NSObject, NSObjectProtocol, NSString,
};
use objc2_ui_kit::{
    NSAttributedStringNSStringDrawing, NSFontAttributeName, NSTextAlignment, UIScrollViewDelegate,
    UITextView, UITextViewDelegate,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{HAlign, Point, Size};

use crate::{GlobalRuntime, Result, catch, from_cgsize, widgets::Widget};

#[derive(Debug)]
pub struct TextBox {
    handle: Widget,
    text_view: Retained<UITextView>,
    delegate: Retained<TextBoxDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl TextBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| unsafe {
            let text_view = UITextView::new(mtm);
            text_view.setEditable(true);
            text_view.setSelectable(true);

            let handle = Widget::from_uiview(parent, Retained::cast_unchecked(text_view.clone()))?;

            let delegate = TextBoxDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            text_view.setDelegate(Some(del_obj));
            Ok(Self {
                handle,
                text_view,
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
        let text = self.text()?;
        catch(|| unsafe {
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
        })
    }

    pub fn preferred_size(&self) -> Result<Size> {
        catch(|| unsafe {
            let font = self.text_view.font();
            let text = NSAttributedString::initWithString_attributes(
                NSAttributedString::alloc(),
                &self.text_view.text(),
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
        })
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        catch(|| crate::from_nsstring(&self.text_view.text()))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let ns = NSString::from_str(s.as_ref());
        catch(|| self.text_view.setText(Some(&ns)))
    }

    pub fn halign(&self) -> Result<HAlign> {
        let align = catch(|| self.text_view.textAlignment())?;
        let align = match align {
            NSTextAlignment::Right => HAlign::Right,
            NSTextAlignment::Center => HAlign::Center,
            NSTextAlignment::Justified => HAlign::Stretch,
            _ => HAlign::Left,
        };
        Ok(align)
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        let align = match align {
            HAlign::Left => NSTextAlignment::Left,
            HAlign::Center => NSTextAlignment::Center,
            HAlign::Right => NSTextAlignment::Right,
            HAlign::Stretch => NSTextAlignment::Justified,
        };
        catch(|| self.text_view.setTextAlignment(align))
    }

    pub fn is_readonly(&self) -> Result<bool> {
        catch(|| !self.text_view.isEditable())
    }

    pub fn set_readonly(&mut self, v: bool) -> Result<()> {
        catch(|| self.text_view.setEditable(!v))
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
    #[name = "WinioTextBoxDelegateUIKit"]
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

    unsafe impl UIScrollViewDelegate for TextBoxDelegate {}

    #[allow(non_snake_case)]
    unsafe impl UITextViewDelegate for TextBoxDelegate {
        #[unsafe(method(textViewDidChange:))]
        fn textViewDidChange(&self, _text_view: &UITextView) {
            self.ivars().changed.signal::<GlobalRuntime>(());
        }
    }
}

impl TextBoxDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
