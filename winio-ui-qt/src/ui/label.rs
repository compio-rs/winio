use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{HAlign, Point, Size};

use crate::{
    GlobalRuntime, Result,
    ui::{QtAlignmentFlag, Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct Label {
    widget: Widget<ffi::QLabel>,
}

#[inherit_methods(from = "self.widget")]
impl Label {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = unsafe { ffi::new_label(parent.as_container().as_qt()) }?;
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        Ok(Self { widget })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        Ok(self.widget.as_ref().text()?.try_into()?)
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.pin_mut().setText(&s.as_ref().try_into()?)?;
        Ok(())
    }

    pub fn halign(&self) -> Result<HAlign> {
        let flag = self.widget.as_ref().alignment()?;
        let align = if flag.contains(QtAlignmentFlag::AlignRight) {
            HAlign::Right
        } else if flag.contains(QtAlignmentFlag::AlignHCenter) {
            HAlign::Center
        } else if flag.contains(QtAlignmentFlag::AlignJustify) {
            HAlign::Stretch
        } else {
            HAlign::Left
        };
        Ok(align)
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        let mut flag = self.widget.as_ref().alignment()?;
        flag &= QtAlignmentFlag::from_bits_retain(0xFFF0);
        match align {
            HAlign::Left => flag |= QtAlignmentFlag::AlignLeft,
            HAlign::Center => flag |= QtAlignmentFlag::AlignHCenter,
            HAlign::Right => flag |= QtAlignmentFlag::AlignRight,
            HAlign::Stretch => flag |= QtAlignmentFlag::AlignJustify,
        }
        self.widget.pin_mut().setAlignment(flag)?;
        Ok(())
    }
}

winio_handle::impl_as_widget!(Label, widget);

impl_static_cast!(ffi::QLabel, ffi::QWidget);

#[derive(Debug)]
pub struct LinkLabel {
    on_click: Box<Callback>,
    label: Label,
    text: String,
    uri: String,
}

#[inherit_methods(from = "self.label")]
impl LinkLabel {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mut label = Label::new(parent)?;
        label.set_halign(HAlign::Center)?;
        let on_click = Box::new(Callback::new());
        unsafe {
            ffi::label_connect_link_activated(
                label.widget.pin_mut(),
                Self::on_click,
                on_click.as_ref() as *const _ as _,
            )?;
        }
        Ok(Self {
            on_click,
            label,
            text: String::new(),
            uri: String::new(),
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    fn refresh_text(&mut self) -> Result<()> {
        let mut text = "<a href=\"".to_string();
        html_escape::encode_unquoted_attribute_to_string(&self.uri, &mut text);
        text.push_str("\">");
        html_escape::encode_text_to_string(&self.text, &mut text);
        text.push_str("</a>");
        self.label.set_text(text)?;
        if self.uri.is_empty() {
            self.label.widget.pin_mut().setOpenExternalLinks(false)?;
        } else {
            self.label.widget.pin_mut().setOpenExternalLinks(true)?;
        }
        Ok(())
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.text.clone())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.text = s.as_ref().to_string();
        self.refresh_text()
    }

    pub fn uri(&self) -> Result<String> {
        Ok(self.uri.clone())
    }

    pub fn set_uri(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.uri = s.as_ref().to_string();
        self.refresh_text()
    }

    fn on_click(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

winio_handle::impl_as_widget!(LinkLabel, label);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/label.hpp");

        type QWidget = crate::ui::QWidget;
        type QLabel;
        type QString = crate::ui::QString;
        type QtAlignmentFlag = crate::ui::QtAlignmentFlag;

        unsafe fn new_label(parent: *mut QWidget) -> Result<UniquePtr<QLabel>>;

        unsafe fn label_connect_link_activated(
            w: Pin<&mut QLabel>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;

        fn alignment(self: &QLabel) -> Result<QtAlignmentFlag>;
        fn setAlignment(self: Pin<&mut QLabel>, flag: QtAlignmentFlag) -> Result<()>;
        fn text(self: &QLabel) -> Result<QString>;
        fn setText(self: Pin<&mut QLabel>, s: &QString) -> Result<()>;

        fn setOpenExternalLinks(self: Pin<&mut QLabel>, v: bool) -> Result<()>;
    }
}
