use winio::prelude::*;

#[derive(Debug)]
pub enum FailableWebView {
    Widget(WebView),
    ErrLabel(Child<TextBox>),
}

impl Failable for FailableWebView {
    type Error = Error;
}

impl FailableWebView {
    pub fn source(&self) -> Result<String> {
        match self {
            FailableWebView::Widget(wv) => wv.source(),
            FailableWebView::ErrLabel(_) => Ok(String::new()),
        }
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        match self {
            FailableWebView::Widget(wv) => wv.set_source(s),
            FailableWebView::ErrLabel(_) => Ok(()),
        }
    }

    pub fn navigate(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.set_source(s)
    }

    pub fn set_html(&mut self, s: impl AsRef<str>) -> Result<()> {
        match self {
            FailableWebView::Widget(wv) => wv.set_html(s),
            FailableWebView::ErrLabel(_) => Ok(()),
        }
    }

    pub fn navigate_to_string(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.set_html(s)
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        match self {
            FailableWebView::Widget(wv) => wv.can_go_forward(),
            FailableWebView::ErrLabel(_) => Ok(false),
        }
    }

    pub fn go_forward(&mut self) -> Result<()> {
        match self {
            FailableWebView::Widget(wv) => wv.go_forward(),
            FailableWebView::ErrLabel(_) => Ok(()),
        }
    }

    pub fn can_go_back(&self) -> Result<bool> {
        match self {
            FailableWebView::Widget(wv) => wv.can_go_back(),
            FailableWebView::ErrLabel(_) => Ok(false),
        }
    }

    pub fn go_back(&mut self) -> Result<()> {
        match self {
            FailableWebView::Widget(wv) => wv.go_back(),
            FailableWebView::ErrLabel(_) => Ok(()),
        }
    }

    pub fn reload(&mut self) -> Result<()> {
        match self {
            FailableWebView::Widget(wv) => wv.reload(),
            FailableWebView::ErrLabel(_) => Ok(()),
        }
    }

    pub fn stop(&mut self) -> Result<()> {
        match self {
            FailableWebView::Widget(wv) => wv.stop(),
            FailableWebView::ErrLabel(_) => Ok(()),
        }
    }
}

impl Visible for FailableWebView {
    fn is_visible(&self) -> Result<bool> {
        match self {
            FailableWebView::Widget(wv) => wv.is_visible(),
            FailableWebView::ErrLabel(lbl) => lbl.is_visible(),
        }
    }

    fn set_visible(&mut self, v: bool) -> Result<()> {
        match self {
            FailableWebView::Widget(wv) => wv.set_visible(v),
            FailableWebView::ErrLabel(lbl) => lbl.set_visible(v),
        }
    }
}

impl Enable for FailableWebView {
    fn is_enabled(&self) -> Result<bool> {
        match self {
            FailableWebView::Widget(wv) => wv.is_enabled(),
            FailableWebView::ErrLabel(lbl) => lbl.is_enabled(),
        }
    }

    fn set_enabled(&mut self, v: bool) -> Result<()> {
        match self {
            FailableWebView::Widget(wv) => wv.set_enabled(v),
            FailableWebView::ErrLabel(lbl) => lbl.set_enabled(v),
        }
    }
}

impl Layoutable for FailableWebView {
    fn loc(&self) -> Result<Point> {
        match self {
            FailableWebView::Widget(wv) => wv.loc(),
            FailableWebView::ErrLabel(lbl) => lbl.loc(),
        }
    }

    fn set_loc(&mut self, v: Point) -> Result<()> {
        match self {
            FailableWebView::Widget(wv) => wv.set_loc(v),
            FailableWebView::ErrLabel(lbl) => lbl.set_loc(v),
        }
    }

    fn size(&self) -> Result<Size> {
        match self {
            FailableWebView::Widget(wv) => wv.size(),
            FailableWebView::ErrLabel(lbl) => lbl.size(),
        }
    }

    fn set_size(&mut self, v: Size) -> Result<()> {
        match self {
            FailableWebView::Widget(wv) => wv.set_size(v),
            FailableWebView::ErrLabel(lbl) => lbl.set_size(v),
        }
    }
}

impl Component for FailableWebView {
    type Error = Error;
    type Event = WebViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = WebViewMessage;

    async fn init(init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        match WebView::init(init.clone(), sender.cast()).await {
            Ok(wv) => Ok(FailableWebView::Widget(wv)),
            Err(e) => {
                let text = if cfg!(windows) {
                    format!(
                        "WebView2 failed to initialize: {}\n\n\
                         This application requires the Microsoft Edge WebView2 Runtime.\n\n\
                         Please download and install the runtime from:\n  \
                         https://aka.ms/webview2installer",
                        e
                    )
                } else {
                    format!("WebView failed to initialize: {}", e)
                };
                init! {
                    lbl: TextBox = (init) => {
                        text: text,
                        readonly: true,
                    }
                }
                Ok(FailableWebView::ErrLabel(lbl))
            }
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        match self {
            FailableWebView::Widget(wv) => wv.start(sender.cast()).await,
            FailableWebView::ErrLabel(_) => std::future::pending().await,
        }
    }
}
