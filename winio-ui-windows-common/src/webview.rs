use std::{cell::RefCell, fmt::Debug, future::Future, rc::Rc};

use futures_util::future::Either;
use winio_handle::{
    AsContainer, AsRawContainer, AsRawWidget, AsWidget, BorrowedContainer, BorrowedWidget,
    RawWidget,
};
use winio_primitive::{Point, Size};

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
    fn init<W: WebViewImpl>(&self, w: &mut W) {
        w.set_visible(self.visible);
        w.set_enabled(self.enabled);
        w.set_loc(self.loc);
        w.set_size(self.size);
        match &self.source {
            LazyInitSource::Url(s) => w.set_source(s),
            LazyInitSource::Html(s) => w.set_html(s),
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait WebViewImpl {
    async fn new(parent: impl AsContainer) -> Self;

    fn is_visible(&self) -> bool;
    fn set_visible(&mut self, v: bool);

    fn is_enabled(&self) -> bool;
    fn set_enabled(&mut self, v: bool);

    fn loc(&self) -> Point;
    fn set_loc(&mut self, v: Point);

    fn size(&self) -> Size;
    fn set_size(&mut self, v: Size);

    fn source(&self) -> String;
    fn set_source(&mut self, s: impl AsRef<str>);
    fn set_html(&mut self, s: impl AsRef<str>);

    fn can_go_forward(&self) -> bool;
    fn go_forward(&mut self);

    fn can_go_back(&self) -> bool;
    fn go_back(&mut self);

    fn reload(&mut self);
    fn stop(&mut self);

    fn wait_navigating(&self) -> impl Future<Output = ()> + 'static + use<Self>;
    fn wait_navigated(&self) -> impl Future<Output = ()> + 'static + use<Self>;
}

enum WebViewInner<W> {
    Params(LazyInitParams),
    Widget(W),
}

impl<W> Debug for WebViewInner<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebViewInner").finish_non_exhaustive()
    }
}

impl<W: WebViewImpl> WebViewInner<W> {
    pub fn is_visible(&self) -> bool {
        match self {
            Self::Params(p) => p.visible,
            Self::Widget(w) => w.is_visible(),
        }
    }

    pub fn set_visible(&mut self, v: bool) {
        match self {
            Self::Params(p) => p.visible = v,
            Self::Widget(w) => w.set_visible(v),
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self {
            Self::Params(p) => p.enabled,
            Self::Widget(w) => w.is_enabled(),
        }
    }

    pub fn set_enabled(&mut self, v: bool) {
        match self {
            Self::Params(p) => p.enabled = v,
            Self::Widget(w) => w.set_enabled(v),
        }
    }

    pub fn loc(&self) -> Point {
        match self {
            Self::Params(p) => p.loc,
            Self::Widget(w) => w.loc(),
        }
    }

    pub fn set_loc(&mut self, v: Point) {
        match self {
            Self::Params(p) => p.loc = v,
            Self::Widget(w) => w.set_loc(v),
        }
    }

    pub fn size(&self) -> Size {
        match self {
            Self::Params(p) => p.size,
            Self::Widget(w) => w.size(),
        }
    }

    pub fn set_size(&mut self, v: Size) {
        match self {
            Self::Params(p) => p.size = v,
            Self::Widget(w) => w.set_size(v),
        }
    }

    pub fn source(&self) -> String {
        match self {
            Self::Params(p) => match &p.source {
                LazyInitSource::Url(s) => s.clone(),
                LazyInitSource::Html(_) => "about:blank".into(),
            },
            Self::Widget(w) => w.source(),
        }
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) {
        match self {
            Self::Params(p) => p.source = LazyInitSource::Url(s.as_ref().into()),
            Self::Widget(w) => w.set_source(s),
        }
    }

    pub fn set_html(&mut self, s: impl AsRef<str>) {
        match self {
            Self::Params(p) => p.source = LazyInitSource::Html(s.as_ref().into()),
            Self::Widget(w) => w.set_html(s),
        }
    }

    pub fn can_go_forward(&self) -> bool {
        match self {
            Self::Params(_) => false,
            Self::Widget(w) => w.can_go_forward(),
        }
    }

    pub fn go_forward(&mut self) {
        match self {
            Self::Params(_) => unreachable!("cannot go forward before initialized"),
            Self::Widget(w) => w.go_forward(),
        }
    }

    pub fn can_go_back(&self) -> bool {
        match self {
            Self::Params(_) => false,
            Self::Widget(w) => w.can_go_back(),
        }
    }

    pub fn go_back(&mut self) {
        match self {
            Self::Params(_) => unreachable!("cannot go back before initialized"),
            Self::Widget(w) => w.go_back(),
        }
    }

    pub fn reload(&mut self) {
        match self {
            Self::Params(_) => unreachable!("cannot reload before initialized"),
            Self::Widget(w) => w.reload(),
        }
    }

    pub fn stop(&mut self) {
        match self {
            Self::Params(_) => unreachable!("cannot stop before initialized"),
            Self::Widget(w) => w.stop(),
        }
    }

    pub fn wait_navigating(&self) -> impl Future<Output = ()> + 'static {
        match self {
            Self::Params(_) => Either::Left(std::future::pending()),
            Self::Widget(w) => Either::Right(w.wait_navigating()),
        }
    }

    pub fn wait_navigated(&self) -> impl Future<Output = ()> + 'static {
        match self {
            Self::Params(_) => Either::Left(std::future::pending()),
            Self::Widget(w) => Either::Right(w.wait_navigated()),
        }
    }
}

impl<W: AsRawWidget> AsRawWidget for WebViewInner<W> {
    fn as_raw_widget(&self) -> RawWidget {
        match self {
            Self::Params(_) => unreachable!("cannot get raw widget before initialized"),
            Self::Widget(w) => w.as_raw_widget(),
        }
    }
}

#[derive(Debug)]
pub struct WebViewLazy<W> {
    inner: Rc<RefCell<WebViewInner<W>>>,
}

impl<W: WebViewImpl + 'static> WebViewLazy<W> {
    pub fn new(parent: impl AsContainer) -> Self {
        let inner = Rc::new(RefCell::new(
            WebViewInner::Params(LazyInitParams::default()),
        ));
        compio::runtime::spawn({
            let inner = inner.clone();
            let parent = parent.as_container().as_raw_container();
            async move {
                let mut w = W::new(unsafe { BorrowedContainer::borrow_raw(parent) }).await;
                let mut inner = inner.borrow_mut();
                if let WebViewInner::Params(p) = &*inner {
                    p.init(&mut w);
                    *inner = WebViewInner::Widget(w);
                }
            }
        })
        .detach();
        Self { inner }
    }
}

impl<W: WebViewImpl> WebViewLazy<W> {
    pub fn is_visible(&self) -> bool {
        self.inner.borrow().is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.inner.borrow_mut().set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.inner.borrow().is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.inner.borrow_mut().set_enabled(v);
    }

    pub fn loc(&self) -> Point {
        self.inner.borrow().loc()
    }

    pub fn set_loc(&mut self, v: Point) {
        self.inner.borrow_mut().set_loc(v);
    }

    pub fn size(&self) -> Size {
        self.inner.borrow().size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.inner.borrow_mut().set_size(v);
    }

    pub fn preferred_size(&self) -> Size {
        Size::zero()
    }

    pub fn source(&self) -> String {
        self.inner.borrow().source()
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) {
        self.inner.borrow_mut().set_source(s);
    }

    pub fn set_html(&mut self, s: impl AsRef<str>) {
        self.inner.borrow_mut().set_html(s);
    }

    pub fn can_go_forward(&self) -> bool {
        self.inner.borrow().can_go_forward()
    }

    pub fn go_forward(&mut self) {
        self.inner.borrow_mut().go_forward();
    }

    pub fn can_go_back(&self) -> bool {
        self.inner.borrow().can_go_back()
    }

    pub fn go_back(&mut self) {
        self.inner.borrow_mut().go_back();
    }

    pub fn reload(&mut self) {
        self.inner.borrow_mut().reload();
    }

    pub fn stop(&mut self) {
        self.inner.borrow_mut().stop();
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

impl<W: AsRawWidget> AsRawWidget for WebViewLazy<W> {
    fn as_raw_widget(&self) -> RawWidget {
        self.inner.borrow().as_raw_widget()
    }
}

impl<W: AsRawWidget> AsWidget for WebViewLazy<W> {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        unsafe { BorrowedWidget::borrow_raw(self.as_raw_widget()) }
    }
}
