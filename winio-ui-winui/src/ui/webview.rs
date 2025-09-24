use std::{future::Future, rc::Rc};

use send_wrapper::SendWrapper;
use windows::{
    Foundation::{TypedEventHandler, Uri},
    core::{HSTRING, Interface},
};
use winio_callback::Callback;
use winio_handle::{AsContainer, AsRawWidget, RawWidget};
use winio_primitive::{Point, Size};
use winio_ui_windows_common::{WebViewImpl, WebViewLazy};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{GlobalRuntime, Widget};

#[derive(Debug)]
pub struct WebViewInner {
    on_navigating: SendWrapper<Rc<Callback>>,
    on_navigated: SendWrapper<Rc<Callback>>,
    handle: Widget,
    view: MUXC::WebView2,
}

impl WebViewImpl for WebViewInner {
    async fn new(parent: impl AsContainer) -> Self {
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
        let view = MUXC::WebView2::new().unwrap();
        view.EnsureCoreWebView2Async().unwrap().await.unwrap();
        let on_navigating = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_navigating = on_navigating.clone();
            view.NavigationStarting(&TypedEventHandler::new(move |_, _| {
                on_navigating.signal::<GlobalRuntime>(());
                Ok(())
            }))
            .unwrap();
        }
        let on_navigated = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_navigated = on_navigated.clone();
            view.NavigationCompleted(&TypedEventHandler::new(move |_, _| {
                on_navigated.signal::<GlobalRuntime>(());
                Ok(())
            }))
            .unwrap();
        }
        Self {
            on_navigating,
            on_navigated,
            handle: Widget::new(parent, view.cast().unwrap()),
            view,
        }
    }

    fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v)
    }

    fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v)
    }

    fn loc(&self) -> Point {
        self.handle.loc()
    }

    fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p)
    }

    fn size(&self) -> Size {
        self.handle.size()
    }

    fn set_size(&mut self, v: Size) {
        self.handle.set_size(v)
    }

    fn source(&self) -> String {
        self.view
            .Source()
            .unwrap()
            .ToString()
            .unwrap()
            .to_string_lossy()
    }

    fn set_source(&mut self, s: impl AsRef<str>) {
        self.view
            .SetSource(&Uri::CreateUri(&HSTRING::from(s.as_ref())).unwrap())
            .unwrap()
    }

    fn set_html(&mut self, s: impl AsRef<str>) {
        self.view
            .NavigateToString(&HSTRING::from(s.as_ref()))
            .unwrap();
    }

    fn can_go_forward(&self) -> bool {
        self.view.CanGoForward().unwrap()
    }

    fn go_forward(&mut self) {
        self.view.GoForward().unwrap();
    }

    fn can_go_back(&self) -> bool {
        self.view.CanGoBack().unwrap()
    }

    fn go_back(&mut self) {
        self.view.GoBack().unwrap();
    }

    fn reload(&mut self) {
        self.view.Reload().unwrap();
    }

    fn stop(&mut self) {
        self.view.CoreWebView2().unwrap().Stop().unwrap();
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

pub type WebView = WebViewLazy<WebViewInner>;
