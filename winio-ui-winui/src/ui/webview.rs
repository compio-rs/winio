use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    Foundation::{TypedEventHandler, Uri},
    core::{HSTRING, Interface},
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{GlobalRuntime, Result, Widget};

#[derive(Debug)]
pub struct WebView {
    on_navigating: SendWrapper<Rc<Callback>>,
    on_navigated: SendWrapper<Rc<Callback>>,
    handle: Widget,
    view: MUXC::WebView2,
}

#[inherit_methods(from = "self.handle")]
impl WebView {
    pub async fn new(parent: impl AsContainer) -> Result<Self> {
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

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, v: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn source(&self) -> Result<String> {
        Ok(self.view.Source()?.ToString()?.to_string_lossy())
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        if s.is_empty() {
            return self.set_html("");
        }
        self.view.SetSource(&Uri::CreateUri(&HSTRING::from(s))?)?;
        Ok(())
    }

    pub fn set_html(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.view.NavigateToString(&HSTRING::from(s.as_ref()))?;
        Ok(())
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        self.view.CanGoForward()
    }

    pub fn go_forward(&mut self) -> Result<()> {
        self.view.GoForward()?;
        Ok(())
    }

    pub fn can_go_back(&self) -> Result<bool> {
        self.view.CanGoBack()
    }

    pub fn go_back(&mut self) -> Result<()> {
        self.view.GoBack()?;
        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        self.view.Reload()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.view.CoreWebView2()?.Stop()?;
        Ok(())
    }

    pub async fn wait_navigating(&self) {
        self.on_navigating.wait().await;
    }

    pub async fn wait_navigated(&self) {
        self.on_navigated.wait().await;
    }
}

winio_handle::impl_as_widget!(WebView, handle);
