use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_foundation::{
    MainThreadMarker, NSObject, NSObjectProtocol, NSString, NSURL, NSURLRequest,
};
use objc2_web_kit::{WKNavigation, WKNavigationDelegate, WKWebView, WKWebViewConfiguration};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime,
    ui::{Widget, from_nsstring},
};

#[derive(Debug)]
pub struct WebView {
    handle: Widget,
    view: Retained<WKWebView>,
    delegate: Retained<WebViewDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl WebView {
    pub fn new(parent: impl AsContainer) -> Self {
        unsafe {
            let parent = parent.as_container();
            let mtm = parent.mtm();

            let frame = parent.frame();
            let config = WKWebViewConfiguration::new(mtm);
            let view =
                WKWebView::initWithFrame_configuration(WKWebView::alloc(mtm), frame, &config);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            let delegate = WebViewDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            view.setNavigationDelegate(Some(del_obj));

            Self {
                handle,
                view,
                delegate,
            }
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool {
        true
    }

    #[allow(clippy::unused_unit)]
    pub fn set_enabled(&mut self, _: bool) {
        ()
    }

    pub fn preferred_size(&self) -> Size {
        Size::zero()
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn source(&self) -> String {
        unsafe {
            self.view
                .URL()
                .and_then(|url| url.absoluteString())
                .map(|s| from_nsstring(&s))
                .unwrap_or_default()
        }
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) {
        let s = s.as_ref();
        if s.is_empty() {
            return self.set_html("");
        }
        unsafe {
            if let Some(url) = NSURL::URLWithString(&NSString::from_str(s)) {
                let req = NSURLRequest::requestWithURL(&url);
                self.view.loadRequest(&req);
            }
        }
    }

    pub fn set_html(&mut self, html: impl AsRef<str>) {
        unsafe {
            self.view
                .loadHTMLString_baseURL(&NSString::from_str(html.as_ref()), None);
        }
    }

    pub fn can_go_forward(&self) -> bool {
        unsafe { self.view.canGoForward() }
    }

    pub fn go_forward(&mut self) {
        unsafe {
            self.view.goForward();
        }
    }

    pub fn can_go_back(&self) -> bool {
        unsafe { self.view.canGoBack() }
    }

    pub fn go_back(&mut self) {
        unsafe {
            self.view.goBack();
        }
    }

    pub fn reload(&mut self) {
        unsafe {
            self.view.reload();
        }
    }

    pub fn stop(&mut self) {
        unsafe {
            self.view.stopLoading();
        }
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
    #[name = "WinioWebViewDelegate"]
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

    #[allow(non_snake_case)]
    unsafe impl WKNavigationDelegate for WebViewDelegate {
        #[unsafe(method(webView:didCommitNavigation:))]
        unsafe fn webView_didCommitNavigation(
            &self,
            _web_view: &WKWebView,
            _navigation: Option<&WKNavigation>,
        ) {
            self.ivars().navigating.signal::<GlobalRuntime>(());
        }

        #[unsafe(method(webView:didFinishNavigation:))]
        unsafe fn webView_didFinishNavigation(
            &self,
            _web_view: &WKWebView,
            _navigation: Option<&WKNavigation>,
        ) {
            self.ivars().navigated.signal::<GlobalRuntime>(());
        }
    }
}

impl WebViewDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
