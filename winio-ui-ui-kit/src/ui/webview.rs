use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Result, catch, ui::Widget};

#[derive(Debug)]
pub struct WebView {
    handle: Widget,
    delegate: Retained<WebViewDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl WebView {
    pub async fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| unsafe {
            let view = objc2_ui_kit::UIView::new(mtm);
            let handle = Widget::from_uiview(parent, Retained::cast_unchecked(view.clone()))?;

            let delegate = WebViewDelegate::new(mtm);

            Ok(Self { handle, delegate })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(true)
    }

    pub fn set_enabled(&mut self, _: bool) -> Result<()> {
        Ok(())
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn source(&self) -> Result<String> {
        Ok(String::new())
    }

    pub fn set_source(&mut self, _s: impl AsRef<str>) -> Result<()> {
        Ok(())
    }

    pub fn set_html(&mut self, _html: impl AsRef<str>) -> Result<()> {
        Ok(())
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        Ok(false)
    }

    pub fn go_forward(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn can_go_back(&self) -> Result<bool> {
        Ok(false)
    }

    pub fn go_back(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn wait_navigating(&self) {
        self.delegate.ivars().navigating.wait().await
    }

    pub async fn wait_navigated(&self) {
        self.delegate.ivars().navigated.wait().await
    }
}

winio_handle::impl_as_widget!(WebView, handle);

#[derive(Debug, Default)]
struct WebViewDelegateIvars {
    navigating: Callback,
    navigated: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioWebViewDelegateUIKit"]
    #[ivars = WebViewDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct WebViewDelegate;

    #[allow(non_snake_case)]
    impl WebViewDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(WebViewDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for WebViewDelegate {}
}

impl WebViewDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
