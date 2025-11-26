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
use winio_handle::AsContainer;
use winio_primitive::{HAlign, Point, Size};

use crate::{
    Error, GlobalRuntime, Result, catch,
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
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.mtm();

        catch(|| unsafe {
            let view = NSTextView::scrollableTextView(mtm);
            let text_view = Retained::cast_unchecked::<NSTextView>(
                view.documentView().ok_or(Error::NullPointer)?,
            );
            text_view.setRichText(false);
            text_view.setEditable(true);
            text_view.setSelectable(true);

            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

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
        })
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        catch(|| from_nsstring(&self.text_view.string()))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        catch(|| self.text_view.setString(&NSString::from_str(s.as_ref())))
    }

    pub fn halign(&self) -> Result<HAlign> {
        let align = catch(|| self.text_view.alignment())?;
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
        catch(|| self.text_view.setAlignment(align))
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
