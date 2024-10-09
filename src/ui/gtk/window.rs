use std::rc::Rc;

use gtk4::{glib::Propagation, prelude::*};

use crate::{AsRawWindow, ColorTheme, Point, RawWindow, Size, ui::Callback};

pub struct Window {
    on_size: Rc<Callback<()>>,
    on_close: Rc<Callback<()>>,
    window: gtk4::Window,
    swindow: gtk4::ScrolledWindow,
    #[allow(dead_code)]
    fixed: gtk4::Fixed,
}

impl Window {
    pub fn new() -> Self {
        let window = gtk4::Window::new();

        let color = window.color();
        let brightness = color.red() * 0.299 + color.green() * 0.587 + color.blue() * 0.114;
        let theme = if brightness > 0.5 {
            ColorTheme::Dark
        } else {
            ColorTheme::Light
        };
        super::COLOR_THEME.set(theme);

        let swindow = gtk4::ScrolledWindow::new();
        let fixed = gtk4::Fixed::new();
        window.set_child(Some(&swindow));
        swindow.set_child(Some(&fixed));

        let on_size = Rc::new(Callback::new());
        let on_close = Rc::new(Callback::new());

        window.connect_default_width_notify({
            let on_size = Rc::downgrade(&on_size);
            move |_| {
                if let Some(on_size) = on_size.upgrade() {
                    on_size.signal(());
                }
            }
        });
        window.connect_default_height_notify({
            let on_size = Rc::downgrade(&on_size);
            move |_| {
                if let Some(on_size) = on_size.upgrade() {
                    on_size.signal(());
                }
            }
        });
        window.connect_state_flags_changed({
            let on_size = Rc::downgrade(&on_size);
            move |_, _| {
                if let Some(on_size) = on_size.upgrade() {
                    on_size.signal(());
                }
            }
        });
        window.connect_close_request({
            let on_close = Rc::downgrade(&on_close);
            move |_| {
                if let Some(on_close) = on_close.upgrade() {
                    if !on_close.signal(()) {
                        return Propagation::Stop;
                    }
                }
                Propagation::Proceed
            }
        });
        window.set_visible(true);
        Self {
            on_size,
            on_close,
            window,
            swindow,
            fixed,
        }
    }

    pub fn loc(&self) -> Point {
        Point::zero()
    }

    pub fn set_loc(&mut self, _p: Point) {}

    pub fn size(&self) -> Size {
        let (_, size) = self.window.preferred_size();
        let (_, width, ..) = self
            .window
            .measure(gtk4::Orientation::Horizontal, size.width());
        let (_, height, ..) = self
            .window
            .measure(gtk4::Orientation::Vertical, size.height());
        Size::new(
            self.window.width().max(width) as f64,
            self.window.height().max(height) as f64,
        )
    }

    pub fn set_size(&mut self, s: Size) {
        self.window.set_default_size(s.width as _, s.height as _);
    }

    pub fn client_size(&self) -> Size {
        let width = self.swindow.width();
        let height = self.swindow.height();
        Size::new(width as _, height as _)
    }

    pub fn text(&self) -> String {
        self.window
            .title()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.window.set_title(Some(s.as_ref()));
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
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> RawWindow {
        self.window.clone()
    }
}
