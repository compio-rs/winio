use crate::{Point, Rect, Size};

/// A trait for types that can fail with an associated error type.
pub trait Failable {
    /// The error type associated.
    type Error;
}

/// Trait for a widget to set visibility.
pub trait Visible: Failable {
    /// If the widget is visible.
    fn is_visible(&self) -> Result<bool, Self::Error>;

    /// Set the visibility.
    fn set_visible(&mut self, v: bool) -> Result<(), Self::Error>;

    /// Show the widget.
    fn show(&mut self) -> Result<(), Self::Error> {
        self.set_visible(true)
    }

    /// Hide the widget.
    fn hide(&mut self) -> Result<(), Self::Error> {
        self.set_visible(false)
    }
}

/// Trait for a widget to enable or disable.
pub trait Enable: Failable {
    /// If the widget is enabled.
    fn is_enabled(&self) -> Result<bool, Self::Error>;

    /// Set if the widget is enabled.
    fn set_enabled(&mut self, v: bool) -> Result<(), Self::Error>;

    /// Enable the widget.
    fn enable(&mut self) -> Result<(), Self::Error> {
        self.set_enabled(true)
    }

    /// Disable the widget.
    fn disable(&mut self) -> Result<(), Self::Error> {
        self.set_enabled(false)
    }
}

/// Common trait for widgets that have a tooltip.
pub trait ToolTip: Failable {
    /// Get the tooltip text of the widget.
    fn tooltip(&self) -> Result<String, Self::Error>;

    /// Set the tooltip text of the widget.
    fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<(), Self::Error>;
}

/// Common trait for widgets that have text.
pub trait TextWidget: Failable {
    /// Get the text of the widget.
    fn text(&self) -> Result<String, Self::Error>;

    /// Set the text of the widget.
    ///
    /// If the widget supports multiline strings, lines are separated with `\n`.
    /// You don't need to handle CRLF.
    fn set_text(&mut self, s: impl AsRef<str>) -> Result<(), Self::Error>;
}

/// Trait for a layoutable widget.
///
/// To create a responsive layout, always set location and size together.
pub trait Layoutable: Failable {
    /// The left top location.
    fn loc(&self) -> Result<Point, Self::Error>;

    /// Move the location.
    fn set_loc(&mut self, p: Point) -> Result<(), Self::Error>;

    /// The size.
    fn size(&self) -> Result<Size, Self::Error>;

    /// Resize.
    fn set_size(&mut self, s: Size) -> Result<(), Self::Error>;

    /// The bounding rectangle.
    fn rect(&self) -> Result<Rect, Self::Error> {
        Ok(Rect::new(self.loc()?, self.size()?))
    }

    /// Set the location and size.
    fn set_rect(&mut self, r: Rect) -> Result<(), Self::Error> {
        self.set_loc(r.origin)?;
        self.set_size(r.size)
    }

    /// The preferred size.
    fn preferred_size(&self) -> Result<Size, Self::Error> {
        Ok(Size::zero())
    }

    /// Min acceptable size.
    fn min_size(&self) -> Result<Size, Self::Error> {
        self.preferred_size()
    }
}
