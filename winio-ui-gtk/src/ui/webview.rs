use std::rc::Rc;

use gtk4::glib::object::Cast;
use inherit_methods_macro::inherit_methods;
use webkit6::prelude::WebViewExt;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Result, ui::Widget};

#[derive(Debug)]
pub struct WebView {
    on_loading: Rc<Callback<()>>,
    on_loaded: Rc<Callback<()>>,
    widget: webkit6::WebView,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl WebView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = webkit6::WebView::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() })?;
        let on_loading = Rc::new(Callback::new());
        widget.connect_resource_load_started({
            let on_loading = on_loading.clone();
            move |_, _, _| {
                on_loading.signal::<GlobalRuntime>(());
            }
        });
        let on_loaded = Rc::new(Callback::new());
        widget.connect_load_changed({
            let on_loaded = on_loaded.clone();
            move |_, _| {
                on_loaded.signal::<GlobalRuntime>(());
            }
        });
        Ok(Self {
            on_loading,
            on_loaded,
            widget,
            handle,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        Ok(Size::zero())
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn source(&self) -> Result<String> {
        Ok(self.widget.uri().map(|s| s.to_string()).unwrap_or_default())
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.load_uri(s.as_ref());
        Ok(())
    }

    pub fn set_html(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.load_html(s.as_ref(), None);
        Ok(())
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        Ok(self.widget.can_go_forward())
    }

    pub fn go_forward(&mut self) -> Result<()> {
        self.widget.go_forward();
        Ok(())
    }

    pub fn can_go_back(&self) -> Result<bool> {
        Ok(self.widget.can_go_back())
    }

    pub fn go_back(&mut self) -> Result<()> {
        self.widget.go_back();
        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        self.widget.reload();
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.widget.stop_loading();
        Ok(())
    }

    pub async fn wait_navigating(&self) {
        self.on_loading.wait().await
    }

    pub async fn wait_navigated(&self) {
        self.on_loaded.wait().await
    }
}

winio_handle::impl_as_widget!(WebView, handle);
