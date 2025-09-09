use std::{path::PathBuf, rc::Rc};

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    Foundation::{TypedEventHandler, Uri},
    core::{HSTRING, Interface},
};
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{GlobalRuntime, Widget};

#[derive(Debug)]
pub struct WebView {
    on_navigate: SendWrapper<Rc<Callback>>,
    handle: Widget,
    view: MUXC::WebView2,
}

#[inherit_methods(from = "self.handle")]
impl WebView {
    pub async fn new(parent: impl AsWindow) -> Self {
        #[cfg(feature = "webview-system")]
        {
            fn add_webview2sdk_path() {
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
        let on_navigate = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_navigate = on_navigate.clone();
            view.NavigationCompleted(&TypedEventHandler::new(move |_, _| {
                on_navigate.signal::<GlobalRuntime>(());
                Ok(())
            }))
            .unwrap();
        }
        Self {
            on_navigate,
            handle: Widget::new(parent, view.cast().unwrap()),
            view,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        Size::zero()
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn source(&self) -> String {
        self.view
            .Source()
            .unwrap()
            .ToString()
            .unwrap()
            .to_string_lossy()
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) {
        self.view
            .SetSource(&Uri::CreateUri(&HSTRING::from(s.as_ref())).unwrap())
            .unwrap()
    }

    pub fn can_go_forward(&self) -> bool {
        self.view.CanGoForward().unwrap()
    }

    pub fn go_forward(&mut self) {
        self.view.GoForward().unwrap();
    }

    pub fn can_go_back(&self) -> bool {
        self.view.CanGoBack().unwrap()
    }

    pub fn go_back(&mut self) {
        self.view.GoBack().unwrap();
    }

    pub async fn wait_navigate(&self) {
        self.on_navigate.wait().await;
    }
}

winio_handle::impl_as_widget!(WebView, handle);
