use std::rc::Rc;

use gtk4::{glib::Propagation, prelude::*};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::{AsContainer, AsWindow, BorrowedContainer, BorrowedWindow};
use winio_primitive::{ColorTheme, Point, Size};

use crate::{GlobalRuntime, Result, Widget};

#[derive(Debug)]
pub struct Window {
    on_size: Rc<Callback<()>>,
    on_close: Rc<Callback<()>>,
    on_theme: Rc<Callback<()>>,
    window: gtk4::Window,
    swindow: gtk4::ScrolledWindow,
    fixed: gtk4::Fixed,
    #[allow(unused)]
    settings: gtk4::Settings,
}

impl Window {
    pub fn new() -> Result<Self> {
        let window = gtk4::Window::new();

        set_color_theme(&window);

        let swindow = gtk4::ScrolledWindow::new();
        swindow.set_hscrollbar_policy(gtk4::PolicyType::External);
        swindow.set_vscrollbar_policy(gtk4::PolicyType::External);
        let fixed = gtk4::Fixed::new();
        window.set_child(Some(&swindow));
        swindow.set_child(Some(&fixed));

        let on_size = Rc::new(Callback::new());
        let on_close = Rc::new(Callback::new());
        let on_theme = Rc::new(Callback::new());

        window.connect_default_width_notify({
            let on_size = on_size.clone();
            move |_| {
                on_size.signal::<GlobalRuntime>(());
            }
        });
        window.connect_default_height_notify({
            let on_size = on_size.clone();
            move |_| {
                on_size.signal::<GlobalRuntime>(());
            }
        });
        window.connect_state_flags_changed({
            let on_size = on_size.clone();
            move |_, _| {
                on_size.signal::<GlobalRuntime>(());
            }
        });
        window.connect_close_request({
            let on_close = on_close.clone();
            move |_| {
                if !on_close.signal::<GlobalRuntime>(()) {
                    return Propagation::Stop;
                }
                Propagation::Proceed
            }
        });
        let settings = gtk4::Settings::for_display(&WidgetExt::display(&window));
        settings.connect_closure("notify::gtk-theme-name", true, {
            let on_theme = on_theme.clone();
            let window = window.clone();
            gtk4::glib::RustClosure::new_local(move |_| {
                set_color_theme(&window);
                on_theme.signal::<GlobalRuntime>(());
                None
            })
        });

        Ok(Self {
            on_size,
            on_close,
            on_theme,
            window,
            swindow,
            fixed,
            settings,
        })
    }

    pub fn is_visible(&self) -> Result<bool> {
        Ok(self.window.get_visible())
    }

    pub fn set_visible(&self, v: bool) -> Result<()> {
        if v {
            self.window.present();
        } else {
            self.window.set_visible(v);
        }
        Ok(())
    }

    pub fn loc(&self) -> Result<Point> {
        Ok(Point::zero())
    }

    pub fn set_loc(&mut self, _p: Point) -> Result<()> {
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        let mut width = self.window.width();
        if width == 0 {
            width = self.window.default_width();
        }
        let mut height = self.window.height();
        if height == 0 {
            height = self.window.default_height();
        }
        Ok(Size::new(width as _, height as _))
    }

    pub fn set_size(&mut self, s: Size) -> Result<()> {
        self.window.set_default_size(s.width as _, s.height as _);
        Ok(())
    }

    pub fn client_size(&self) -> Result<Size> {
        let mut width = self.swindow.width();
        if width == 0 {
            width = self.window.default_width();
        }
        let mut height = self.swindow.height();
        if height == 0 {
            height = self.window.default_height();
        }
        Ok(Size::new(width as _, height as _))
    }

    pub fn text(&self) -> Result<String> {
        Ok(self
            .window
            .title()
            .map(|s| s.to_string())
            .unwrap_or_default())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.window.set_title(Some(s.as_ref()));
        Ok(())
    }

    pub async fn wait_size(&self) {
        self.on_size.wait().await
    }

    pub async fn wait_move(&self) {
        std::future::pending().await
    }

    pub async fn wait_close(&self) {
        self.on_close.wait().await
    }

    pub async fn wait_theme_changed(&self) {
        self.on_theme.wait().await
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        BorrowedWindow::gtk(&self.window)
    }
}

impl AsContainer for Window {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::gtk(&self.fixed)
    }
}

fn set_color_theme(w: &gtk4::Window) {
    let color = w.color();
    let brightness = color.red() * 0.299 + color.green() * 0.587 + color.blue() * 0.114;
    let theme = if brightness > 0.5 {
        ColorTheme::Dark
    } else {
        ColorTheme::Light
    };
    super::COLOR_THEME.set(Some(theme));
}

#[derive(Debug)]
pub struct View {
    fixed: gtk4::Fixed,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl View {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let fixed = gtk4::Fixed::new();
        let handle = Widget::new(parent, unsafe { fixed.clone().unsafe_cast() })?;
        Ok(Self { fixed, handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;
}

winio_handle::impl_as_widget!(View, handle);

impl AsContainer for View {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::gtk(&self.fixed)
    }
}
