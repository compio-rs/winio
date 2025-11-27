use std::{future::Future, rc::Rc};

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    Foundation::{TypedEventHandler, Uri},
    core::{HSTRING, Interface},
};
use winio_callback::Callback;
use winio_handle::{AsContainer, AsRawWidget, RawWidget};
use winio_primitive::{Point, Size};
use winio_ui_windows_common::{WebViewErrLabelImpl, WebViewImpl, WebViewLazy};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{GlobalRuntime, Result, TextBox, Widget};

#[derive(Debug)]
pub struct WebViewInner {
    on_navigating: SendWrapper<Rc<Callback>>,
    on_navigated: SendWrapper<Rc<Callback>>,
    handle: Widget,
    view: MUXC::WebView2,
}

impl WebViewImpl for WebViewInner {
    async fn new(parent: impl AsContainer) -> Result<Self> {
        #[cfg(feature = "webview-system")]
        {
            fn add_webview2sdk_path() {
                use std::path::PathBuf;

                use windows::{
                    Win32::{
                        System::LibraryLoader::{
                            AddDllDirectory, LOAD_LIBRARY_SEARCH_SYSTEM32,
                            LOAD_LIBRARY_SEARCH_USER_DIRS, SetDefaultDllDirectories,
                        },
                        UI::Shell::{CSIDL_WINDOWS, SHGetSpecialFolderPathW},
                    },
                    core::PCWSTR,
                };

                unsafe {
                    SetDefaultDllDirectories(
                        LOAD_LIBRARY_SEARCH_USER_DIRS | LOAD_LIBRARY_SEARCH_SYSTEM32,
                    )
                    .ok();

                    let mut buffer = [0u16; 260];
                    if SHGetSpecialFolderPathW(None, &mut buffer, CSIDL_WINDOWS as _, false)
                        .ok()
                        .is_ok()
                    {
                        let windir =
                            widestring::U16CStr::from_ptr_str(buffer.as_ptr()).to_os_string();
                        let dlldir = PathBuf::from(windir).join(r"SystemApps\Shared\WebView2SDK");

                        if let Ok(dlldir) = widestring::U16CString::from_os_str(&dlldir) {
                            AddDllDirectory(PCWSTR(dlldir.as_ptr()));
                        }
                    }
                }
            }

            use std::sync::Once;

            static ADD_PATH: Once = Once::new();

            ADD_PATH.call_once(add_webview2sdk_path);
        }
        let view = MUXC::WebView2::new()?;
        view.EnsureCoreWebView2Async()?.await?;
        let on_navigating = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_navigating = on_navigating.clone();
            view.NavigationStarting(&TypedEventHandler::new(move |_, _| {
                on_navigating.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }
        let on_navigated = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_navigated = on_navigated.clone();
            view.NavigationCompleted(&TypedEventHandler::new(move |_, _| {
                on_navigated.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }
        Ok(Self {
            on_navigating,
            on_navigated,
            handle: Widget::new(parent, view.cast()?)?,
            view,
        })
    }

    fn is_visible(&self) -> Result<bool> {
        self.handle.is_visible()
    }

    fn set_visible(&mut self, v: bool) -> Result<()> {
        self.handle.set_visible(v)
    }

    fn is_enabled(&self) -> Result<bool> {
        self.handle.is_enabled()
    }

    fn set_enabled(&mut self, v: bool) -> Result<()> {
        self.handle.set_enabled(v)
    }

    fn loc(&self) -> Result<Point> {
        self.handle.loc()
    }

    fn set_loc(&mut self, v: Point) -> Result<()> {
        self.handle.set_loc(v)
    }

    fn size(&self) -> Result<Size> {
        self.handle.size()
    }

    fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)
    }

    fn source(&self) -> Result<String> {
        Ok(self.view.Source()?.ToString()?.to_string_lossy())
    }

    fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        if s.is_empty() {
            return self.set_html("");
        }
        self.view.SetSource(&Uri::CreateUri(&HSTRING::from(s))?)?;
        Ok(())
    }

    fn set_html(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.view.NavigateToString(&HSTRING::from(s.as_ref()))?;
        Ok(())
    }

    fn can_go_forward(&self) -> Result<bool> {
        self.view.CanGoForward()
    }

    fn go_forward(&mut self) -> Result<()> {
        self.view.GoForward()?;
        Ok(())
    }

    fn can_go_back(&self) -> Result<bool> {
        self.view.CanGoBack()
    }

    fn go_back(&mut self) -> Result<()> {
        self.view.GoBack()?;
        Ok(())
    }

    fn reload(&mut self) -> Result<()> {
        self.view.Reload()?;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.view.CoreWebView2()?.Stop()?;
        Ok(())
    }

    fn wait_navigating(&self) -> impl Future<Output = ()> + 'static + use<> {
        let on_navigating = self.on_navigating.clone();
        async move {
            on_navigating.wait().await;
        }
    }

    fn wait_navigated(&self) -> impl Future<Output = ()> + 'static + use<> {
        let on_navigated = self.on_navigated.clone();
        async move {
            on_navigated.wait().await;
        }
    }
}

impl AsRawWidget for WebViewInner {
    fn as_raw_widget(&self) -> RawWidget {
        self.handle.as_raw_widget()
    }
}

#[derive(Debug)]
pub struct WebViewErrLabelInner {
    handle: TextBox,
}

#[inherit_methods(from = "self.handle")]
impl WebViewErrLabelImpl for WebViewErrLabelInner {
    fn new(parent: impl AsContainer) -> Result<Self> {
        let mut handle = TextBox::new(parent)?;
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

    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.handle")]
impl AsRawWidget for WebViewErrLabelInner {
    fn as_raw_widget(&self) -> RawWidget;
}

pub type WebView = WebViewLazy<WebViewInner, WebViewErrLabelInner>;
