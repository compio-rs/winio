use std::{cell::RefCell, fmt::Debug, future::Future, rc::Rc};

use compio_log::error;
use futures_util::future::Either;
use winio_handle::{
    AsContainer, AsRawContainer, AsRawWidget, AsWidget, BorrowedContainer, BorrowedWidget,
    RawWidget,
};
use winio_primitive::{Point, Size};

use crate::Result;

enum LazyInitSource {
    Url(String),
    Html(String),
}

struct LazyInitParams {
    visible: bool,
    enabled: bool,
    loc: Point,
    size: Size,
    source: LazyInitSource,
}

impl Default for LazyInitParams {
    fn default() -> Self {
        Self {
            visible: true,
            enabled: true,
            loc: Point::zero(),
            size: Size::zero(),
            source: LazyInitSource::Url(String::new()),
        }
    }
}

impl LazyInitParams {
    fn init<W: WebViewImpl>(&self, w: &mut W) -> Result<()> {
        w.set_visible(self.visible)?;
        w.set_enabled(self.enabled)?;
        w.set_loc(self.loc)?;
        w.set_size(self.size)?;
        match &self.source {
            LazyInitSource::Url(s) => w.set_source(s),
            LazyInitSource::Html(s) => w.set_html(s),
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait WebViewImpl: Sized {
    async fn new(parent: impl AsContainer) -> Result<Self>;

    fn is_visible(&self) -> Result<bool>;
    fn set_visible(&mut self, v: bool) -> Result<()>;

    fn is_enabled(&self) -> Result<bool>;
    fn set_enabled(&mut self, v: bool) -> Result<()>;

    fn loc(&self) -> Result<Point>;
    fn set_loc(&mut self, v: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;
    fn set_size(&mut self, v: Size) -> Result<()>;

    fn source(&self) -> Result<String>;
    fn set_source(&mut self, s: impl AsRef<str>) -> Result<()>;
    fn set_html(&mut self, s: impl AsRef<str>) -> Result<()>;

    fn can_go_forward(&self) -> Result<bool>;
    fn go_forward(&mut self) -> Result<()>;

    fn can_go_back(&self) -> Result<bool>;
    fn go_back(&mut self) -> Result<()>;

    fn reload(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;

    fn wait_navigating(&self) -> impl Future<Output = ()> + 'static + use<Self>;
    fn wait_navigated(&self) -> impl Future<Output = ()> + 'static + use<Self>;
}

pub trait WebViewErrLabelImpl: Sized {
    fn new(parent: impl AsContainer) -> Result<Self>;

    fn is_visible(&self) -> Result<bool>;
    fn set_visible(&mut self, v: bool) -> Result<()>;

    fn is_enabled(&self) -> Result<bool>;
    fn set_enabled(&mut self, v: bool) -> Result<()>;

    fn loc(&self) -> Result<Point>;
    fn set_loc(&mut self, v: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;
    fn set_size(&mut self, v: Size) -> Result<()>;

    fn text(&self) -> Result<String>;
    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

enum WebViewInner<W, L> {
    Params(LazyInitParams),
    Widget(W),
    ErrLabel(L),
}

impl<W, L> Debug for WebViewInner<W, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebViewInner").finish_non_exhaustive()
    }
}

impl<W: WebViewImpl, L: WebViewErrLabelImpl> WebViewInner<W, L> {
    pub fn is_visible(&self) -> Result<bool> {
        match self {
            Self::Params(p) => Ok(p.visible),
            Self::Widget(w) => w.is_visible(),
            Self::ErrLabel(l) => l.is_visible(),
        }
    }

    pub fn set_visible(&mut self, v: bool) -> Result<()> {
        match self {
            Self::Params(p) => {
                p.visible = v;
                Ok(())
            }
            Self::Widget(w) => w.set_visible(v),
            Self::ErrLabel(l) => l.set_visible(v),
        }
    }

    pub fn is_enabled(&self) -> Result<bool> {
        match self {
            Self::Params(p) => Ok(p.enabled),
            Self::Widget(w) => w.is_enabled(),
            Self::ErrLabel(l) => l.is_enabled(),
        }
    }

    pub fn set_enabled(&mut self, v: bool) -> Result<()> {
        match self {
            Self::Params(p) => {
                p.enabled = v;
                Ok(())
            }
            Self::Widget(w) => w.set_enabled(v),
            Self::ErrLabel(l) => l.set_enabled(v),
        }
    }

    pub fn loc(&self) -> Result<Point> {
        match self {
            Self::Params(p) => Ok(p.loc),
            Self::Widget(w) => w.loc(),
            Self::ErrLabel(l) => l.loc(),
        }
    }

    pub fn set_loc(&mut self, v: Point) -> Result<()> {
        match self {
            Self::Params(p) => {
                p.loc = v;
                Ok(())
            }
            Self::Widget(w) => w.set_loc(v),
            Self::ErrLabel(l) => l.set_loc(v),
        }
    }

    pub fn size(&self) -> Result<Size> {
        match self {
            Self::Params(p) => Ok(p.size),
            Self::Widget(w) => w.size(),
            Self::ErrLabel(l) => l.size(),
        }
    }

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        match self {
            Self::Params(p) => {
                p.size = v;
                Ok(())
            }
            Self::Widget(w) => w.set_size(v),
            Self::ErrLabel(l) => l.set_size(v),
        }
    }

    pub fn source(&self) -> Result<String> {
        match self {
            Self::Params(p) => match &p.source {
                LazyInitSource::Url(s) => Ok(s.clone()),
                LazyInitSource::Html(_) => Ok(String::new()),
            },
            Self::Widget(w) => w.source(),
            Self::ErrLabel(_) => Ok(String::new()),
        }
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        match self {
            Self::Params(p) => {
                p.source = LazyInitSource::Url(s.as_ref().into());
                Ok(())
            }
            Self::Widget(w) => w.set_source(s),
            Self::ErrLabel(_) => Ok(()),
        }
    }

    pub fn set_html(&mut self, s: impl AsRef<str>) -> Result<()> {
        match self {
            Self::Params(p) => {
                p.source = LazyInitSource::Html(s.as_ref().into());
                Ok(())
            }
            Self::Widget(w) => w.set_html(s),
            Self::ErrLabel(_) => Ok(()),
        }
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        match self {
            Self::Params(_) | Self::ErrLabel(_) => Ok(false),
            Self::Widget(w) => w.can_go_forward(),
        }
    }

    pub fn go_forward(&mut self) -> Result<()> {
        match self {
            Self::Params(_) | Self::ErrLabel(_) => Ok(()),
            Self::Widget(w) => w.go_forward(),
        }
    }

    pub fn can_go_back(&self) -> Result<bool> {
        match self {
            Self::Params(_) | Self::ErrLabel(_) => Ok(false),
            Self::Widget(w) => w.can_go_back(),
        }
    }

    pub fn go_back(&mut self) -> Result<()> {
        match self {
            Self::Params(_) | Self::ErrLabel(_) => Ok(()),
            Self::Widget(w) => w.go_back(),
        }
    }

    pub fn reload(&mut self) -> Result<()> {
        match self {
            Self::Params(_) | Self::ErrLabel(_) => Ok(()),
            Self::Widget(w) => w.reload(),
        }
    }

    pub fn stop(&mut self) -> Result<()> {
        match self {
            Self::Params(_) | Self::ErrLabel(_) => Ok(()),
            Self::Widget(w) => w.stop(),
        }
    }

    pub fn wait_navigating(&self) -> impl Future<Output = ()> + 'static {
        match self {
            Self::Params(_) | Self::ErrLabel(_) => Either::Left(std::future::pending()),
            Self::Widget(w) => Either::Right(w.wait_navigating()),
        }
    }

    pub fn wait_navigated(&self) -> impl Future<Output = ()> + 'static {
        match self {
            Self::Params(_) | Self::ErrLabel(_) => Either::Left(std::future::pending()),
            Self::Widget(w) => Either::Right(w.wait_navigated()),
        }
    }
}

impl<W: AsRawWidget, L: AsRawWidget> AsRawWidget for WebViewInner<W, L> {
    fn as_raw_widget(&self) -> RawWidget {
        match self {
            Self::Params(_) => unreachable!("cannot get raw widget before initialized"),
            Self::Widget(w) => w.as_raw_widget(),
            Self::ErrLabel(l) => l.as_raw_widget(),
        }
    }
}

#[derive(Debug)]
pub struct WebViewLazy<W, L> {
    inner: Rc<RefCell<WebViewInner<W, L>>>,
}

impl<W: WebViewImpl + 'static, L: WebViewErrLabelImpl + 'static> WebViewLazy<W, L> {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let inner = Rc::new(RefCell::new(
            WebViewInner::Params(LazyInitParams::default()),
        ));
        compio::runtime::spawn({
            let inner = inner.clone();
            let parent = parent.as_container().as_raw_container();
            async move {
                let w = W::new(unsafe { BorrowedContainer::borrow_raw(parent.clone()) }).await;
                let mut inner = inner.borrow_mut();
                if let WebViewInner::Params(p) = &*inner {
                    let init = || -> Result<_> {
                        let mut w = w?;
                        p.init(&mut w)?;
                        Ok(w)
                    };
                    match init() {
                        Ok(w) => {
                            *inner = WebViewInner::Widget(w);
                        }
                        Err(e) => {
                            let creat = || -> Result<_> {
                                let mut l = L::new(unsafe { BorrowedContainer::borrow_raw(parent) })?;
                                l.set_text(format!(
                                    "WebView2 failed to initialize: {}\n\n\
                                     This application requires the Microsoft Edge WebView2 Runtime.\n\n\
                                     Please download and install the runtime from:\n  \
                                     https://aka.ms/webview2installer",
                                    e
                                ))?;
                                Ok(l)
                            };
                            match creat() {
                                Ok(l) => {
                                    *inner = WebViewInner::ErrLabel(l);
                                }
                                Err(_e) => {
                                    error!("Failed to create WebView error label: {}", _e);
                                }
                            }
                        }
                    }
                }
            }
        })
        .detach();
        Ok(Self { inner })
    }
}

impl<W: WebViewImpl, L: WebViewErrLabelImpl> WebViewLazy<W, L> {
    pub fn is_visible(&self) -> Result<bool> {
        self.inner.borrow().is_visible()
    }

    pub fn set_visible(&mut self, v: bool) -> Result<()> {
        self.inner.borrow_mut().set_visible(v)
    }

    pub fn is_enabled(&self) -> Result<bool> {
        self.inner.borrow().is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) -> Result<()> {
        self.inner.borrow_mut().set_enabled(v)
    }

    pub fn loc(&self) -> Result<Point> {
        self.inner.borrow().loc()
    }

    pub fn set_loc(&mut self, v: Point) -> Result<()> {
        self.inner.borrow_mut().set_loc(v)
    }

    pub fn size(&self) -> Result<Size> {
        self.inner.borrow().size()
    }

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.inner.borrow_mut().set_size(v)
    }

    pub fn preferred_size(&self) -> Result<Size> {
        Ok(Size::zero())
    }

    pub fn source(&self) -> Result<String> {
        self.inner.borrow().source()
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.inner.borrow_mut().set_source(s)
    }

    pub fn set_html(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.inner.borrow_mut().set_html(s)
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        self.inner.borrow().can_go_forward()
    }

    pub fn go_forward(&mut self) -> Result<()> {
        self.inner.borrow_mut().go_forward()
    }

    pub fn can_go_back(&self) -> Result<bool> {
        self.inner.borrow().can_go_back()
    }

    pub fn go_back(&mut self) -> Result<()> {
        self.inner.borrow_mut().go_back()
    }

    pub fn reload(&mut self) -> Result<()> {
        self.inner.borrow_mut().reload()
    }

    pub fn stop(&mut self) -> Result<()> {
        self.inner.borrow_mut().stop()
    }

    pub async fn wait_navigating(&self) {
        let fut = self.inner.borrow().wait_navigating();
        fut.await
    }

    pub async fn wait_navigated(&self) {
        let fut = self.inner.borrow().wait_navigated();
        fut.await
    }
}

impl<W: AsRawWidget, L: AsRawWidget> AsRawWidget for WebViewLazy<W, L> {
    fn as_raw_widget(&self) -> RawWidget {
        self.inner.borrow().as_raw_widget()
    }
}

impl<W: AsRawWidget, L: AsRawWidget> AsWidget for WebViewLazy<W, L> {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        unsafe { BorrowedWidget::borrow_raw(self.as_raw_widget()) }
    }
}
