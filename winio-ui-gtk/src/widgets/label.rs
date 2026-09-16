use gtk4::{
    glib::object::Cast,
    pango::{AttrFontDesc, AttrList},
    prelude::{FixedExt, WidgetExt},
};
use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Font, HAlign, Point, Size};

use crate::{
    Error, Image, Result,
    widgets::{Widget, desc_to_font, font_desc_from_attrs, font_to_desc},
};

#[derive(Debug)]
pub struct Label {
    widget: LabelImpl,
    handle: Widget,
}

#[derive(Debug)]
enum LabelImpl {
    Label(gtk4::Label),
    Image(gtk4::Picture),
}

impl LabelImpl {
    fn widget(&self) -> &gtk4::Widget {
        match self {
            Self::Label(widget) => widget.upcast_ref(),
            Self::Image(widget) => widget.upcast_ref(),
        }
    }

    fn as_label(&self) -> Option<&gtk4::Label> {
        match self {
            Self::Label(widget) => Some(widget),
            Self::Image(_) => None,
        }
    }

    fn as_label_mut(&mut self) -> Option<&mut gtk4::Label> {
        match self {
            Self::Label(widget) => Some(widget),
            Self::Image(_) => None,
        }
    }
}

#[inherit_methods(from = "self.handle")]
impl Label {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = LabelImpl::Label(gtk4::Label::new(None));
        let handle = Widget::new(parent, widget.widget().clone())?;
        Ok(Self { widget, handle })
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
        Ok(self
            .widget
            .as_label()
            .map(|widget| widget.text().to_string())
            .unwrap_or_default())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        let widget = match &mut self.widget {
            LabelImpl::Label(widget) => {
                widget.set_text(s);
                None
            }
            LabelImpl::Image(_) => Some(LabelImpl::Label(gtk4::Label::new(Some(s)))),
        };
        if let Some(widget) = widget {
            self.replace_widget(widget)?;
        }
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub fn set_image(&mut self, image: &Image) -> Result<()> {
        let widget = match &mut self.widget {
            LabelImpl::Image(widget) => {
                widget.set_paintable(Some(image.texture()));
                None
            }
            LabelImpl::Label(_) => {
                let widget = gtk4::Picture::new();
                widget.set_paintable(Some(image.texture()));
                Some(LabelImpl::Image(widget))
            }
        };
        if let Some(widget) = widget {
            self.replace_widget(widget)?;
        }
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub fn halign(&self) -> Result<HAlign> {
        let Some(widget) = self.widget.as_label() else {
            return Ok(HAlign::Center);
        };
        let align = widget.xalign();
        let align = if align == 0.0 {
            HAlign::Left
        } else if align == 1.0 {
            HAlign::Right
        } else {
            HAlign::Center
        };
        Ok(align)
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        if let Some(widget) = self.widget.as_label_mut() {
            let align = match align {
                HAlign::Left => 0.0,
                HAlign::Right => 1.0,
                _ => 0.5,
            };
            widget.set_xalign(align);
        }
        Ok(())
    }

    pub fn font(&self) -> Result<Font> {
        let Some(widget) = self.widget.as_label() else {
            return Ok(Font::default());
        };
        let desc = font_desc_from_attrs(widget.attributes().as_ref())
            .or_else(|| widget.pango_context().font_description())
            .unwrap_or_default();
        Ok(desc_to_font(&desc))
    }

    pub fn set_font(&mut self, font: Font) -> Result<()> {
        if let Some(widget) = self.widget.as_label_mut() {
            let attr_list = AttrList::new();
            attr_list.insert(AttrFontDesc::new(&font_to_desc(&font)));
            widget.set_attributes(Some(&attr_list));
        }
        self.handle.reset_preferred_size();
        Ok(())
    }

    fn replace_widget(&mut self, widget: LabelImpl) -> Result<()> {
        let old = self.widget.widget().clone();
        let parent = old
            .parent()
            .and_then(|parent| parent.downcast::<gtk4::Fixed>().ok())
            .ok_or(Error::CastFailed)?;
        let (x, y) = parent.child_position(&old);
        let mut handle = Widget::new(BorrowedContainer::gtk(&parent), widget.widget().clone())?;
        handle.set_loc(Point::new(x, y))?;
        handle.set_visible(old.get_visible())?;
        handle.set_enabled(old.get_sensitive())?;
        handle.set_tooltip(old.tooltip_text().unwrap_or_default())?;
        self.handle = handle;
        self.widget = widget;
        Ok(())
    }
}

winio_handle::impl_as_widget!(Label, handle);
