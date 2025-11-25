use std::{cell::RefCell, future::Future, io, rc::Rc};

use inherit_methods_macro::inherit_methods;
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
    Win32::Foundation::{E_FAIL, HWND, RECT},
    core::{HRESULT, PCWSTR, Ref, Result as WinResult, implement},
};
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use winio_callback::Callback;
use winio_handle::{AsContainer, AsRawWidget, RawWidget};
use winio_primitive::{Point, Rect, Size};
use winio_ui_windows_common::{CoTaskMemPtr, WebViewErrLabelImpl, WebViewImpl, WebViewLazy};

use crate::{
    Result,
    ui::{TextBox, fix_crlf, with_u16c},
};

#[derive(Debug)]
pub struct WebViewInner {
    host: ICoreWebView2Controller,
    view: ICoreWebView2,
    navigating: Rc<Callback>,
    navigated: Rc<Callback>,
}

impl WebViewInner {
    fn dpi(&self) -> Result<f64> {
        unsafe {
            let hwnd = self.host.ParentWindow()?;
            Ok(GetDpiForWindow(hwnd.0) as f64 / 96.0)
        }
    }

    fn rect(&self) -> Result<Rect> {
        let rect = unsafe { self.host.Bounds() }?;
        Ok(Rect::new(
            Point::new(rect.left as _, rect.top as _),
            Size::new((rect.right - rect.left) as _, (rect.bottom - rect.top) as _),
        ) / self.dpi()?)
    }

    fn set_rect(&mut self, r: Rect) -> Result<()> {
        let r = r * self.dpi()?;
        unsafe {
            self.host.SetBounds(RECT {
                left: r.origin.x as _,
                top: r.origin.y as _,
                right: (r.origin.x + r.size.width) as _,
                bottom: (r.origin.y + r.size.height) as _,
            })?;
        }
        Ok(())
    }
}

impl WebViewImpl for WebViewInner {
    async fn new(parent: impl AsContainer) -> Result<Self> {
        let (tx, rx) = local_sync::oneshot::channel();
        let hwnd = parent.as_container().as_win32();
        unsafe {
            CreateCoreWebView2Environment(&CreateEnvHandler::create(move |env| {
                let env = env?;
                let env = env.ok()?;
                env.CreateCoreWebView2Controller(
                    HWND(hwnd),
                    &CreateControllerHandler::create(move |host| {
                        let host = host?;
                        let host = host.ok()?;
                        let view = host.CoreWebView2()?;
                        tx.send((host.clone(), view)).ok();
                        Ok(())
                    }),
                )?;
                Ok(())
            }))?;
        }
        let (host, view) = rx
            .await
            .map_err(|_| io::Error::from_raw_os_error(E_FAIL.0))?;
        let navigating = Rc::new(Callback::new());
        unsafe {
            let navigating = navigating.clone();
            view.NavigationStarting(&NavStartingHandler::create(move |_, _| {
                navigating.signal::<()>(());
                Ok(())
            }))?;
        }
        let navigated = Rc::new(Callback::new());
        unsafe {
            let navigated = navigated.clone();
            view.NavigationCompleted(&NavCompletedHandler::create(move |_, _| {
                navigated.signal::<()>(());
                Ok(())
            }))?;
        }
        Ok(Self {
            host,
            view,
            navigating,
            navigated,
        })
    }

    fn is_visible(&self) -> Result<bool> {
        unsafe { Ok(self.host.IsVisible()?.as_bool()) }
    }

    fn set_visible(&mut self, v: bool) -> Result<()> {
        unsafe {
            self.host.SetIsVisible(v)?;
            Ok(())
        }
    }

    fn is_enabled(&self) -> Result<bool> {
        Ok(true)
    }

    fn set_enabled(&mut self, _: bool) -> Result<()> {
        Ok(())
    }

    fn loc(&self) -> Result<Point> {
        Ok(self.rect()?.origin)
    }

    fn set_loc(&mut self, p: Point) -> Result<()> {
        let mut rect = self.rect()?;
        rect.origin = p;
        self.set_rect(rect)
    }

    fn size(&self) -> Result<Size> {
        Ok(self.rect()?.size)
    }

    fn set_size(&mut self, v: Size) -> Result<()> {
        let mut rect = self.rect()?;
        rect.size = v;
        self.set_rect(rect)
    }

    fn source(&self) -> Result<String> {
        unsafe {
            let source = CoTaskMemPtr::new(self.view.Source()?.0);
            Ok(PCWSTR(source.as_ptr())
                .to_string()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
        }
    }

    fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        if s.is_empty() {
            return self.set_html("");
        }
        with_u16c(s, |s| unsafe {
            self.view.Navigate(PCWSTR(s.as_ptr()))?;
            Ok(())
        })
    }

    fn set_html(&mut self, s: impl AsRef<str>) -> Result<()> {
        with_u16c(s.as_ref(), |s| unsafe {
            self.view.NavigateToString(PCWSTR(s.as_ptr()))?;
            Ok(())
        })
    }

    fn can_go_forward(&self) -> Result<bool> {
        unsafe { Ok(self.view.CanGoForward()?.as_bool()) }
    }

    fn go_forward(&mut self) -> Result<()> {
        unsafe {
            self.view.GoForward()?;
            Ok(())
        }
    }

    fn can_go_back(&self) -> Result<bool> {
        unsafe { Ok(self.view.CanGoBack()?.as_bool()) }
    }

    fn go_back(&mut self) -> Result<()> {
        unsafe {
            self.view.GoBack()?;
            Ok(())
        }
    }

    fn reload(&mut self) -> Result<()> {
        unsafe {
            self.view.Reload()?;
            Ok(())
        }
    }

    fn stop(&mut self) -> Result<()> {
        unsafe {
            self.view.Stop()?;
            Ok(())
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

#[derive(Debug)]
pub struct WebViewErrLabelInner {
    handle: TextBox,
}

#[inherit_methods(from = "self.handle")]
impl WebViewErrLabelImpl for WebViewErrLabelInner {
    fn new(parent: impl AsContainer) -> Result<Self> {
        let mut handle = TextBox::new_raw(parent)?;
        handle.set_readonly(true)?;
        Ok(Self { handle })
    }

    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;

    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;

    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, v: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;

    fn text(&self) -> Result<String> {
        Ok(self.handle.text()?.replace("\r\n", "\n"))
    }

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.handle.set_text(fix_crlf(s.as_ref()))
    }
}

#[inherit_methods(from = "self.handle")]
impl AsRawWidget for WebViewErrLabelInner {
    fn as_raw_widget(&self) -> RawWidget;
}

pub type WebView = WebViewLazy<WebViewInner, WebViewErrLabelInner>;

#[implement(
    ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
    Agile = false
)]
struct CreateEnvHandler<F>
where
    F: FnOnce(WinResult<Ref<ICoreWebView2Environment>>) -> WinResult<()> + 'static,
{
    f: RefCell<Option<F>>,
}

impl<F> CreateEnvHandler<F>
where
    F: FnOnce(WinResult<Ref<ICoreWebView2Environment>>) -> WinResult<()> + 'static,
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
    F: FnOnce(WinResult<Ref<ICoreWebView2Environment>>) -> WinResult<()> + 'static,
{
    fn Invoke(
        &self,
        errorcode: HRESULT,
        createdenvironment: Ref<ICoreWebView2Environment>,
    ) -> WinResult<()> {
        let f = self.f.borrow_mut().take();
        if let Some(f) = f {
            f(errorcode.map(|| createdenvironment))
        } else {
            Ok(())
        }
    }
}

#[implement(
    ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
    Agile = false
)]
struct CreateControllerHandler<F>
where
    F: FnOnce(WinResult<Ref<ICoreWebView2Controller>>) -> WinResult<()> + 'static,
{
    f: RefCell<Option<F>>,
}

impl<F> CreateControllerHandler<F>
where
    F: FnOnce(WinResult<Ref<ICoreWebView2Controller>>) -> WinResult<()> + 'static,
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
    F: FnOnce(WinResult<Ref<ICoreWebView2Controller>>) -> WinResult<()> + 'static,
{
    fn Invoke(
        &self,
        errorcode: HRESULT,
        createdcontroller: Ref<ICoreWebView2Controller>,
    ) -> WinResult<()> {
        let f = self.f.borrow_mut().take();
        if let Some(f) = f {
            f(errorcode.map(|| createdcontroller))
        } else {
            Ok(())
        }
    }
}

#[implement(ICoreWebView2NavigationStartingEventHandler, Agile = false)]
struct NavStartingHandler<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationStartingEventArgs>) -> WinResult<()>
        + 'static,
{
    f: F,
}

impl<F> NavStartingHandler<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationStartingEventArgs>) -> WinResult<()>
        + 'static,
{
    pub fn create(f: F) -> ICoreWebView2NavigationStartingEventHandler {
        Self { f }.into()
    }
}

impl<F> ICoreWebView2NavigationStartingEventHandler_Impl for NavStartingHandler_Impl<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationStartingEventArgs>) -> WinResult<()>
        + 'static,
{
    fn Invoke(
        &self,
        sender: Ref<ICoreWebView2>,
        args: Ref<ICoreWebView2NavigationStartingEventArgs>,
    ) -> WinResult<()> {
        (self.f)(sender, args)
    }
}

#[implement(ICoreWebView2NavigationCompletedEventHandler, Agile = false)]
struct NavCompletedHandler<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationCompletedEventArgs>) -> WinResult<()>
        + 'static,
{
    f: F,
}

impl<F> NavCompletedHandler<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationCompletedEventArgs>) -> WinResult<()>
        + 'static,
{
    pub fn create(f: F) -> ICoreWebView2NavigationCompletedEventHandler {
        Self { f }.into()
    }
}

impl<F> ICoreWebView2NavigationCompletedEventHandler_Impl for NavCompletedHandler_Impl<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationCompletedEventArgs>) -> WinResult<()>
        + 'static,
{
    fn Invoke(
        &self,
        sender: Ref<ICoreWebView2>,
        args: Ref<ICoreWebView2NavigationCompletedEventArgs>,
    ) -> WinResult<()> {
        (self.f)(sender, args)
    }
}
