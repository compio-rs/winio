use std::{
    io,
    rc::{Rc, Weak},
};

use gtk4::{glib::Propagation, prelude::*};

use crate::{AsContainer, Callback, ColorTheme, Container, Point, Size};

pub struct Window {
    window: gtk4::Window,
    fixed: gtk4::Fixed,
    on_size: Callback<()>,
    on_close: Callback<()>,
}

impl Window {
    pub fn new() -> io::Result<Rc<Self>> {
        let window = gtk4::Window::new();
        let fixed = gtk4::Fixed::new();
        window.set_child(Some(&fixed));
        Ok(Rc::new_cyclic(|this: &Weak<Self>| {
            window.connect_default_width_notify({
                let this = this.clone();
                move |_| {
                    if let Some(this) = this.upgrade() {
                        this.on_size.signal(());
                    }
                }
            });
            window.connect_default_height_notify({
                let this = this.clone();
                move |_| {
                    if let Some(this) = this.upgrade() {
                        this.on_size.signal(());
                    }
                }
            });
            window.connect_state_flags_changed({
                let this = this.clone();
                move |_, _| {
                    if let Some(this) = this.upgrade() {
                        this.on_size.signal(());
                    }
                }
            });
            window.connect_close_request({
                let this = this.clone();
                move |_| {
                    if let Some(this) = this.upgrade() {
                        if !this.on_close.signal(()) {
                            return Propagation::Stop;
                        }
                    }
                    Propagation::Proceed
                }
            });
            window.set_visible(true);
            Self {
                window,
                fixed,
                on_size: Callback::new(),
                on_close: Callback::new(),
            }
        }))
    }

    pub(crate) fn as_window(&self) -> &gtk4::Window {
        &self.window
    }

    pub fn color_theme(&self) -> ColorTheme {
        let color = self.window.color();
        let brightness = color.red() * 0.299 + color.green() * 0.587 + color.blue() * 0.114;
        if brightness > 0.5 {
            ColorTheme::Dark
        } else {
            ColorTheme::Light
        }
    }

    pub fn loc(&self) -> io::Result<Point> {
        Ok(Point::zero())
    }

    pub fn set_loc(&self, _p: Point) -> io::Result<()> {
        Ok(())
    }

    pub fn size(&self) -> io::Result<Size> {
        let (_, size) = self.window.preferred_size();
        let (_, width, ..) = self
            .window
            .measure(gtk4::Orientation::Horizontal, size.width());
        let (_, height, ..) = self
            .window
            .measure(gtk4::Orientation::Vertical, size.height());
        Ok(Size::new(
            self.window.width().max(width) as f64,
            self.window.height().max(height) as f64,
        ))
    }

    pub fn set_size(&self, s: Size) -> io::Result<()> {
        self.window.set_default_size(s.width as _, s.height as _);
        Ok(())
    }

    pub fn client_size(&self) -> io::Result<Size> {
        let width = self.fixed.width();
        let height = self.fixed.height();
        Ok(Size::new(width as _, height as _))
    }

    pub fn text(&self) -> io::Result<String> {
        Ok(self
            .window
            .title()
            .map(|s| s.to_string())
            .unwrap_or_default())
    }

    pub fn set_text(&self, s: impl AsRef<str>) -> io::Result<()> {
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
}

impl AsContainer for Window {
    fn as_container(&self) -> Container {
        Container::Fixed(self.fixed.clone())
    }
}
