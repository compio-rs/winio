use crate::{Point, Rect, Size};

/// Trait for a layoutable widget.
pub trait Layoutable {
    /// The left top location.
    fn loc(&self) -> Point;

    /// Move the location.
    fn set_loc(&mut self, p: Point);

    /// The size.
    fn size(&self) -> Size;

    /// Resize.
    fn set_size(&mut self, s: Size);

    /// The bounding rectangle.
    fn rect(&self) -> Rect {
        Rect::new(self.loc(), self.size())
    }

    /// Set the location and size.
    fn set_rect(&mut self, r: Rect) {
        self.set_loc(r.origin);
        self.set_size(r.size);
    }
}
