use std::{cell::RefCell, future::Future, rc::Rc};

use webview2::{
    CreateCoreWebView2Environment, ICoreWebView2, ICoreWebView2Controller,
    ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
    ICoreWebView2CreateCoreWebView2ControllerCompletedHandler_Impl,
    ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
    ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler_Impl, ICoreWebView2Environment,
    ICoreWebView2NavigationCompletedEventArgs, ICoreWebView2NavigationCompletedEventHandler,
    ICoreWebView2NavigationCompletedEventHandler_Impl, ICoreWebView2NavigationStartingEventArgs,
    ICoreWebView2NavigationStartingEventHandler, ICoreWebView2NavigationStartingEventHandler_Impl,
};
use windows::{
    Win32::Foundation::{HWND, RECT},
    core::{HRESULT, PCWSTR, Ref, Result, implement},
};
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use winio_callback::Callback;
use winio_handle::{AsContainer, AsRawWidget, RawWidget};
use winio_primitive::{Point, Rect, Size};
use winio_ui_windows_common::{CoTaskMemPtr, WebViewImpl, WebViewLazy};

use crate::ui::with_u16c;

#[derive(Debug)]
pub struct WebViewInner {
    host: ICoreWebView2Controller,
    view: ICoreWebView2,
    navigating: Rc<Callback>,
    navigated: Rc<Callback>,
}

impl WebViewInner {
    fn dpi(&self) -> f64 {
        unsafe {
            let hwnd = self.host.ParentWindow().unwrap();
            GetDpiForWindow(hwnd.0) as f64 / 96.0
        }
    }

    fn rect(&self) -> Rect {
        let rect = unsafe { self.host.Bounds() }.unwrap();
        Rect::new(
            Point::new(rect.left as _, rect.top as _),
            Size::new((rect.right - rect.left) as _, (rect.bottom - rect.top) as _),
        ) / self.dpi()
    }

    #[allow(clippy::missing_transmute_annotations)]
    fn set_rect(&mut self, r: Rect) {
        let r = r * self.dpi();
        unsafe {
            self.host
                .SetBounds(RECT {
                    left: r.origin.x as _,
                    top: r.origin.y as _,
                    right: (r.origin.x + r.size.width) as _,
                    bottom: (r.origin.y + r.size.height) as _,
                })
                .unwrap();
        }
    }
}

impl WebViewImpl for WebViewInner {
    async fn new(parent: impl AsContainer) -> Self {
        let (tx, rx) = local_sync::oneshot::channel();
        let hwnd = parent.as_container().as_win32();
        unsafe {
            CreateCoreWebView2Environment(&CreateEnvHandler::create(move |env| {
                let env = env?;
                let env = env.unwrap();
                env.CreateCoreWebView2Controller(
                    HWND(hwnd),
                    &CreateControllerHandler::create(move |host| {
                        let host = host?;
                        let host = host.unwrap();
                        let view = host.CoreWebView2()?;
                        tx.send((host.clone(), view)).ok();
                        Ok(())
                    }),
                )?;
                Ok(())
            }))
            .unwrap();
        }
        let (host, view) = rx.await.unwrap();
        let navigating = Rc::new(Callback::new());
        unsafe {
            let navigating = navigating.clone();
            view.NavigationStarting(&NavStartingHandler::create(move |_, _| {
                navigating.signal::<()>(());
                Ok(())
            }))
            .unwrap();
        }
        let navigated = Rc::new(Callback::new());
        unsafe {
            let navigated = navigated.clone();
            view.NavigationCompleted(&NavCompletedHandler::create(move |_, _| {
                navigated.signal::<()>(());
                Ok(())
            }))
            .unwrap();
        }
        Self {
            host,
            view,
            navigating,
            navigated,
        }
    }

    fn is_visible(&self) -> bool {
        unsafe { self.host.IsVisible().unwrap().as_bool() }
    }

    fn set_visible(&mut self, v: bool) {
        unsafe {
            self.host.SetIsVisible(v).unwrap();
        }
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn set_enabled(&mut self, _: bool) {}

    fn loc(&self) -> Point {
        self.rect().origin
    }

    fn set_loc(&mut self, p: Point) {
        let mut rect = self.rect();
        rect.origin = p;
        self.set_rect(rect);
    }

    fn size(&self) -> Size {
        self.rect().size
    }

    fn set_size(&mut self, v: Size) {
        let mut rect = self.rect();
        rect.size = v;
        self.set_rect(rect);
    }

    fn source(&self) -> String {
        unsafe {
            let source = CoTaskMemPtr::new(self.view.Source().unwrap().0);
            PCWSTR(source.as_ptr()).to_string().unwrap()
        }
    }

    fn set_source(&mut self, s: impl AsRef<str>) {
        let s = s.as_ref();
        if s.is_empty() {
            return self.set_html("");
        }
        with_u16c(s, |s| unsafe {
            self.view.Navigate(PCWSTR(s.as_ptr())).unwrap();
        })
    }

    fn set_html(&mut self, s: impl AsRef<str>) {
        with_u16c(s.as_ref(), |s| unsafe {
            self.view.NavigateToString(PCWSTR(s.as_ptr())).unwrap();
        })
    }

    fn can_go_forward(&self) -> bool {
        unsafe { self.view.CanGoForward().unwrap().as_bool() }
    }

    fn go_forward(&mut self) {
        unsafe {
            self.view.GoForward().unwrap();
        }
    }

    fn can_go_back(&self) -> bool {
        unsafe { self.view.CanGoBack().unwrap().as_bool() }
    }

    fn go_back(&mut self) {
        unsafe {
            self.view.GoBack().unwrap();
        }
    }

    fn reload(&mut self) {
        unsafe {
            self.view.Reload().unwrap();
        }
    }

    fn stop(&mut self) {
        unsafe {
            self.view.Stop().unwrap();
        }
    }

    fn wait_navigating(&self) -> impl Future<Output = ()> + 'static + use<> {
        let navigating = self.navigating.clone();
        async move {
            navigating.wait().await;
        }
    }

    fn wait_navigated(&self) -> impl Future<Output = ()> + 'static + use<> {
        let navigated = self.navigated.clone();
        async move {
            navigated.wait().await;
        }
    }
}

impl AsRawWidget for WebViewInner {
    fn as_raw_widget(&self) -> RawWidget {
        unimplemented!("cannot get HWND from WebView2")
    }
}

pub type WebView = WebViewLazy<WebViewInner>;

#[implement(ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler)]
struct CreateEnvHandler<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2Environment>>) -> Result<()> + 'static,
{
    f: RefCell<Option<F>>,
}

impl<F> CreateEnvHandler<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2Environment>>) -> Result<()> + 'static,
{
    pub fn create(f: F) -> ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler {
        Self {
            f: RefCell::new(Some(f)),
        }
        .into()
    }
}

impl<F> ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler_Impl for CreateEnvHandler_Impl<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2Environment>>) -> Result<()> + 'static,
{
    fn Invoke(
        &self,
        errorcode: HRESULT,
        createdenvironment: Ref<ICoreWebView2Environment>,
    ) -> Result<()> {
        let f = self.f.borrow_mut().take();
        if let Some(f) = f {
            f(errorcode.map(|| createdenvironment))
        } else {
            Ok(())
        }
    }
}

#[implement(ICoreWebView2CreateCoreWebView2ControllerCompletedHandler)]
struct CreateControllerHandler<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2Controller>>) -> Result<()> + 'static,
{
    f: RefCell<Option<F>>,
}

impl<F> CreateControllerHandler<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2Controller>>) -> Result<()> + 'static,
{
    pub fn create(f: F) -> ICoreWebView2CreateCoreWebView2ControllerCompletedHandler {
        Self {
            f: RefCell::new(Some(f)),
        }
        .into()
    }
}

impl<F> ICoreWebView2CreateCoreWebView2ControllerCompletedHandler_Impl
    for CreateControllerHandler_Impl<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2Controller>>) -> Result<()> + 'static,
{
    fn Invoke(
        &self,
        errorcode: HRESULT,
        createdcontroller: Ref<ICoreWebView2Controller>,
    ) -> Result<()> {
        let f = self.f.borrow_mut().take();
        if let Some(f) = f {
            f(errorcode.map(|| createdcontroller))
        } else {
            Ok(())
        }
    }
}

#[implement(ICoreWebView2NavigationStartingEventHandler)]
struct NavStartingHandler<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationStartingEventArgs>) -> Result<()>
        + 'static,
{
    f: F,
}

impl<F> NavStartingHandler<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationStartingEventArgs>) -> Result<()>
        + 'static,
{
    pub fn create(f: F) -> ICoreWebView2NavigationStartingEventHandler {
        Self { f }.into()
    }
}

impl<F> ICoreWebView2NavigationStartingEventHandler_Impl for NavStartingHandler_Impl<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationStartingEventArgs>) -> Result<()>
        + 'static,
{
    fn Invoke(
        &self,
        sender: Ref<ICoreWebView2>,
        args: Ref<ICoreWebView2NavigationStartingEventArgs>,
    ) -> Result<()> {
        (self.f)(sender, args)
    }
}

#[implement(ICoreWebView2NavigationCompletedEventHandler)]
struct NavCompletedHandler<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationCompletedEventArgs>) -> Result<()>
        + 'static,
{
    f: F,
}

impl<F> NavCompletedHandler<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationCompletedEventArgs>) -> Result<()>
        + 'static,
{
    pub fn create(f: F) -> ICoreWebView2NavigationCompletedEventHandler {
        Self { f }.into()
    }
}

impl<F> ICoreWebView2NavigationCompletedEventHandler_Impl for NavCompletedHandler_Impl<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationCompletedEventArgs>) -> Result<()>
        + 'static,
{
    fn Invoke(
        &self,
        sender: Ref<ICoreWebView2>,
        args: Ref<ICoreWebView2NavigationCompletedEventArgs>,
    ) -> Result<()> {
        (self.f)(sender, args)
    }
}
