use std::{cell::Cell, ptr::NonNull, rc::Rc};

use cookie::Cookie;
use inherit_methods_macro::inherit_methods;
use objc2::{
    AnyThread, DeclaredClass, MainThreadOnly, Message, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::{AnyObject, Bool, ProtocolObject},
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSError, NSHTTPCookie, NSJSONSerialization, NSJSONWritingOptions,
    NSObject, NSObjectProtocol, NSString, NSURL, NSURLRequest, NSUTF8StringEncoding,
};
use objc2_ui_kit::{
    UIAlertAction, UIAlertActionStyle, UIAlertController, UIAlertControllerStyle, UITextField,
    UIViewController,
};
use objc2_web_kit::{
    WKFrameInfo, WKNavigation, WKNavigationDelegate, WKUIDelegate, WKWebViewConfiguration,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winio_ui_apple_common::{cookie_from_ns, cookie_to_ns};

use crate::{Error, GlobalRuntime, Result, Widget, catch, first_ui_window_scene, from_nsstring};

mod generated;
use generated::*;

#[derive(Debug)]
pub struct WebView {
    handle: Widget,
    view: Retained<WKWebView>,
    config: Retained<WKWebViewConfiguration>,
    delegate: Retained<WebViewDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl WebView {
    pub async fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let parent_view = parent.as_ui_kit();
        let mtm = parent_view.mtm();

        catch(|| unsafe {
            let frame = parent_view.frame();
            let config = WKWebViewConfiguration::new(mtm);
            let view =
                WKWebView::initWithFrame_configuration(WKWebView::alloc(mtm), frame, &config);
            let handle = Widget::from_uiview(parent, Retained::cast_unchecked(view.clone()))?;

            let delegate = WebViewDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            view.setNavigationDelegate(Some(del_obj));
            let del_obj = ProtocolObject::from_ref(&*delegate);
            view.setUIDelegate(Some(del_obj));

            Ok(Self {
                handle,
                view,
                config,
                delegate,
            })
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
        catch(|| unsafe {
            self.view
                .URL()
                .and_then(|url| url.absoluteString())
                .map(|s| from_nsstring(&s))
                .unwrap_or_default()
        })
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        if s.is_empty() {
            return self.set_html("");
        }

        catch(|| {
            let url = NSURL::URLWithString(&NSString::from_str(s)).ok_or(Error::NullPointer)?;
            let req = NSURLRequest::requestWithURL(&url);
            unsafe { self.view.loadRequest(&req) };
            Ok(())
        })
        .flatten()
    }

    pub fn set_html(&mut self, html: impl AsRef<str>) -> Result<()> {
        catch(|| unsafe {
            self.view
                .loadHTMLString_baseURL(&NSString::from_str(html.as_ref()), None);
        })
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        catch(|| unsafe { self.view.canGoForward() })
    }

    pub fn go_forward(&mut self) -> Result<()> {
        catch(|| unsafe {
            self.view.goForward();
        })
    }

    pub fn can_go_back(&self) -> Result<bool> {
        catch(|| unsafe { self.view.canGoBack() })
    }

    pub fn go_back(&mut self) -> Result<()> {
        catch(|| unsafe {
            self.view.goBack();
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        catch(|| unsafe {
            self.view.reload();
        })
    }

    pub fn stop(&mut self) -> Result<()> {
        catch(|| unsafe {
            self.view.stopLoading();
        })
    }

    pub async fn wait_navigating(&self) {
        self.delegate.ivars().navigating.wait().await
    }

    pub async fn wait_navigated(&self) {
        self.delegate.ivars().navigated.wait().await
    }

    pub async fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
        let rx = catch(|| {
            let (tx, rx) = local_sync::oneshot::channel();
            let tx = Rc::new(Cell::new(Some(tx)));
            let handler = move |cookies: NonNull<NSArray<NSHTTPCookie>>| {
                if let Some(tx) = tx.take() {
                    tx.send(unsafe { cookies.as_ref() }.retain()).ok();
                }
            };
            let block = block2::StackBlock::new(handler);
            unsafe {
                self.config
                    .websiteDataStore()
                    .httpCookieStore()
                    .getAllCookies(&block);
            }
            rx
        })?;
        let array = rx.await?;
        catch(|| {
            let mut cookies = vec![];
            for cookie in array {
                cookies.push(cookie_from_ns(&cookie)?);
            }
            Ok(cookies)
        })
        .flatten()
    }

    pub async fn set_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        let rx = catch(|| {
            let (tx, rx) = local_sync::oneshot::channel();
            let tx = Rc::new(Cell::new(Some(tx)));
            let handler = move || {
                if let Some(tx) = tx.take() {
                    tx.send(()).ok();
                }
            };
            let block = block2::StackBlock::new(handler);
            let ns_cookie = cookie_to_ns(c)?;
            unsafe {
                self.config
                    .websiteDataStore()
                    .httpCookieStore()
                    .setCookie_completionHandler(&ns_cookie, Some(&block));
            }
            Ok(rx)
        })
        .flatten()?;
        rx.await?;
        Ok(())
    }

    pub async fn delete_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        let rx = catch(|| {
            let (tx, rx) = local_sync::oneshot::channel();
            let tx = Rc::new(Cell::new(Some(tx)));
            let handler = move || {
                if let Some(tx) = tx.take() {
                    tx.send(()).ok();
                }
            };
            let block = block2::StackBlock::new(handler);
            let ns_cookie = cookie_to_ns(c)?;
            unsafe {
                self.config
                    .websiteDataStore()
                    .httpCookieStore()
                    .deleteCookie_completionHandler(&ns_cookie, Some(&block));
            }
            Ok(rx)
        })
        .flatten()?;
        rx.await?;
        Ok(())
    }

    pub async fn run_javascript(&self, js: impl AsRef<str>) -> Result<String> {
        let rx = catch(|| unsafe {
            let (tx, rx) = local_sync::oneshot::channel();
            let tx = Rc::new(Cell::new(Some(tx)));
            let handler = move |result: *mut AnyObject, error: *mut NSError| {
                let res = if error.is_null() {
                    Ok(if result.is_null() {
                        None
                    } else {
                        Some((&*result).retain())
                    })
                } else {
                    Err(Error::NS(Some((&*error).retain())))
                };
                if let Some(tx) = tx.take() {
                    tx.send(res).ok();
                }
            };
            let block = block2::StackBlock::new(handler);
            self.view.evaluateJavaScript_completionHandler(
                &NSString::from_str(js.as_ref()),
                Some(&block),
            );
            Ok(rx)
        })
        .flatten()?;
        let Some(result) = rx.await?? else {
            return Ok(String::new());
        };
        catch(|| {
            let data = unsafe {
                NSJSONSerialization::dataWithJSONObject_options_error(
                    &result,
                    NSJSONWritingOptions(0),
                )?
            };
            let data =
                NSString::initWithData_encoding(NSString::alloc(), &data, NSUTF8StringEncoding);
            data.map(|s| from_nsstring(&s)).ok_or(Error::NullPointer)
        })
        .flatten()
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

    #[allow(non_snake_case)]
    unsafe impl WKUIDelegate for WebViewDelegate {
        #[unsafe(method(webView:runJavaScriptAlertPanelWithMessage:initiatedByFrame:completionHandler:))]
        unsafe fn webView_runJavaScriptAlertPanelWithMessage_initiatedByFrame_completionHandler(
            &self,
            web_view: &WKWebView,
            message: &NSString,
            frame: &WKFrameInfo,
            completion_handler: &block2::DynBlock<dyn Fn()>,
        ) {
            let mtm = self.mtm();
            let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
                None,
                Some(message),
                UIAlertControllerStyle::Alert,
                mtm,
            );
            let action = UIAlertAction::actionWithTitle_style_handler(
                Some(&NSString::from_str("OK")),
                UIAlertActionStyle::Default,
                None,
                mtm,
            );
            alert.addAction(&action);
            if let Some(controller) = Self::root_controller() {
                controller.presentViewController_animated_completion(
                    &alert,
                    true,
                    Some(completion_handler),
                );
            } else {
                completion_handler.call(());
            }
        }

        #[unsafe(method(webView:runJavaScriptConfirmPanelWithMessage:initiatedByFrame:completionHandler:))]
        unsafe fn webView_runJavaScriptConfirmPanelWithMessage_initiatedByFrame_completionHandler(
            &self,
            web_view: &WKWebView,
            message: &NSString,
            frame: &WKFrameInfo,
            completion_handler: &block2::DynBlock<dyn Fn(Bool)>,
        ) {
            let mtm = self.mtm();
            let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
                None,
                Some(message),
                UIAlertControllerStyle::Alert,
                mtm,
            );
            let handler = completion_handler.copy();
            let handler2 = handler.clone();
            let cancel_action = UIAlertAction::actionWithTitle_style_handler(
                Some(&NSString::from_str("Cancel")),
                UIAlertActionStyle::Cancel,
                Some(&block2::StackBlock::new(move |_| {
                    handler2.call((Bool::new(false),));
                })),
                mtm,
            );
            let ok_action = UIAlertAction::actionWithTitle_style_handler(
                Some(&NSString::from_str("OK")),
                UIAlertActionStyle::Default,
                Some(&block2::StackBlock::new(move |_| {
                    handler.call((Bool::new(true),));
                })),
                mtm,
            );
            alert.addAction(&cancel_action);
            alert.addAction(&ok_action);
            if let Some(controller) = Self::root_controller() {
                controller.presentViewController_animated_completion(
                    &alert,
                    true,
                    None,
                );
            } else {
                completion_handler.call((Bool::new(false),));
            }
        }

        #[unsafe(method(webView:runJavaScriptTextInputPanelWithPrompt:defaultText:initiatedByFrame:completionHandler:))]
        unsafe fn webView_runJavaScriptTextInputPanelWithPrompt_defaultText_initiatedByFrame_completionHandler(
            &self,
            web_view: &WKWebView,
            prompt: &NSString,
            default_text: Option<&NSString>,
            frame: &WKFrameInfo,
            completion_handler: &block2::DynBlock<dyn Fn(*mut NSString)>,
        ) {
            let mtm = self.mtm();
            let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
                None,
                Some(prompt),
                UIAlertControllerStyle::Alert,
                mtm,
            );
            let default_text = default_text.map(|s| s.retain());
            let handler = block2::StackBlock::new(move |text_field: NonNull<UITextField>| {
                unsafe { text_field.as_ref() }.setText(default_text.as_deref());
            });
            alert.addTextFieldWithConfigurationHandler(Some(&handler));
            let handler = completion_handler.copy();
            let handler2 = handler.clone();
            let cancel_action = UIAlertAction::actionWithTitle_style_handler(
                Some(&NSString::from_str("Cancel")),
                UIAlertActionStyle::Cancel,
                Some(&block2::StackBlock::new(move |_| {
                    handler2.call((std::ptr::null_mut(),));
                })),
                mtm,
            );
            let alert2 = alert.clone();
            let ok_action = UIAlertAction::actionWithTitle_style_handler(
                Some(&NSString::from_str("OK")),
                UIAlertActionStyle::Default,
                Some(&block2::StackBlock::new(move |_| {
                    let text_field = alert2.textFields().map(|arr| arr.objectAtIndex(0));
                    let text = text_field.and_then(|tf| tf.text());
                    if let Some(text) = text {
                        handler.call((Retained::into_raw(text),));
                    } else {
                        handler.call((std::ptr::null_mut(),));
                    }
                })),
                mtm,
            );
            alert.addAction(&cancel_action);
            alert.addAction(&ok_action);
            if let Some(controller) = Self::root_controller() {
                controller.presentViewController_animated_completion(
                    &alert,
                    true,
                    None,
                );
            } else {
                completion_handler.call((std::ptr::null_mut(),));
            }
        }
    }
}

impl WebViewDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }

    fn root_controller() -> Option<Retained<UIViewController>> {
        first_ui_window_scene()
            .ok()
            .flatten()
            .and_then(|scene| scene.keyWindow())
            .and_then(|wnd| wnd.rootViewController())
    }
}
