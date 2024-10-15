use std::num::NonZero;

use taffy::{
    Style, TaffyTree,
    prelude::{auto, length, percent},
};

use crate::{HAlign, Margin, Orient, Point, Rect, Size, VAlign};

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

    /// The preferred size.
    fn preferred_size(&self) -> Size {
        Size::zero()
    }
}

struct LayoutChild<'a> {
    widget: &'a mut dyn Layoutable,
    width: Option<f64>,
    height: Option<f64>,
    margin: Margin,
    grow: bool,
    halign: HAlign,
    valign: VAlign,
    column: usize,
    row: usize,
    column_span: NonZero<usize>,
    row_span: NonZero<usize>,
}

impl<'a> LayoutChild<'a> {
    pub fn new(widget: &'a mut dyn Layoutable) -> Self {
        Self {
            widget,
            width: None,
            height: None,
            margin: Margin::zero(),
            grow: false,
            halign: HAlign::Stretch,
            valign: VAlign::Stretch,
            column: 0,
            row: 0,
            column_span: NonZero::new(1).unwrap(),
            row_span: NonZero::new(1).unwrap(),
        }
    }
}

/// Builder of a layoutable child.
pub struct LayoutChildBuilder<'a, 'b> {
    child: LayoutChild<'a>,
    children: &'b mut Vec<LayoutChild<'a>>,
}

impl<'a, 'b> LayoutChildBuilder<'a, 'b> {
    /// Specify the child width.
    pub fn width(mut self, v: f64) -> Self {
        self.child.width = Some(v);
        self
    }

    /// Specify the child height.
    pub fn height(mut self, v: f64) -> Self {
        self.child.height = Some(v);
        self
    }

    /// Specify the child size.
    pub fn size(self, s: Size) -> Self {
        self.width(s.width).height(s.height)
    }

    /// Margin of the child.
    pub fn margin(mut self, m: Margin) -> Self {
        self.child.margin = m;
        self
    }

    /// Whether to fill the remain space of the container.
    pub fn grow(mut self, v: bool) -> Self {
        self.child.grow = v;
        self
    }

    /// Horizontal alignment in the available area.
    pub fn halign(mut self, v: HAlign) -> Self {
        self.child.halign = v;
        self
    }

    /// Vertical alignment in the available area.
    pub fn valign(mut self, v: VAlign) -> Self {
        self.child.valign = v;
        self
    }

    /// The column index in the grid.
    pub fn column(mut self, c: usize) -> Self {
        self.child.column = c;
        self
    }

    /// The row index in the grid.
    pub fn row(mut self, r: usize) -> Self {
        self.child.row = r;
        self
    }

    /// The column span in the grid.
    pub fn column_span(mut self, s: NonZero<usize>) -> Self {
        self.child.column_span = s;
        self
    }

    /// The row span in the grid.
    pub fn row_span(mut self, s: NonZero<usize>) -> Self {
        self.child.row_span = s;
        self
    }

    /// Add the child to the container.
    pub fn finish(self) {
        self.children.push(self.child);
    }
}

/// A stacked layout container.
pub struct StackPanel<'a> {
    children: Vec<LayoutChild<'a>>,
    orient: Orient,
    loc: Point,
    size: Size,
}

impl<'a> StackPanel<'a> {
    /// Create [`StackPanel`] with orientation.
    pub fn new(orient: Orient) -> Self {
        Self {
            children: vec![],
            orient,
            loc: Point::zero(),
            size: Size::zero(),
        }
    }

    /// Push a child into the panel.
    pub fn push<'b>(&'b mut self, widget: &'a mut dyn Layoutable) -> LayoutChildBuilder<'a, 'b> {
        LayoutChildBuilder {
            child: LayoutChild::new(widget),
            children: &mut self.children,
        }
    }

    fn render(&mut self) {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        let mut nodes = vec![];
        for child in &self.children {
            let mut style = Style::default();
            style.size.width = match child.width {
                Some(w) => length(w as f32),
                None => match (self.orient, child.halign, child.grow) {
                    (Orient::Vertical, HAlign::Stretch, _) => percent(1.0),
                    (Orient::Horizontal, _, true) => auto(),
                    _ => length(child.widget.preferred_size().width as f32),
                },
            };
            style.size.height = match child.height {
                Some(h) => length(h as f32),
                None => match (self.orient, child.valign, child.grow) {
                    (Orient::Horizontal, VAlign::Stretch, _) => percent(1.0),
                    (Orient::Vertical, _, true) => auto(),
                    _ => length(child.widget.preferred_size().height as f32),
                },
            };
            style.margin = taffy::Rect {
                left: length(child.margin.left as f32),
                right: length(child.margin.right as f32),
                top: length(child.margin.top as f32),
                bottom: length(child.margin.bottom as f32),
            };
            match self.orient {
                Orient::Horizontal => {
                    if matches!(child.valign, VAlign::Top | VAlign::Center) {
                        style.margin.bottom = auto();
                    }
                    if matches!(child.valign, VAlign::Bottom | VAlign::Center) {
                        style.margin.top = auto();
                    }
                }
                Orient::Vertical => {
                    if matches!(child.halign, HAlign::Left | HAlign::Center) {
                        style.margin.right = auto();
                    }
                    if matches!(child.halign, HAlign::Right | HAlign::Center) {
                        style.margin.left = auto();
                    }
                }
            }
            if child.grow {
                style.flex_grow = 1.0
            }
            let node = tree.new_leaf(style).unwrap();
            nodes.push(node);
        }
        let root = tree
            .new_with_children(
                Style {
                    size: taffy::Size::from_percent(1.0, 1.0),
                    flex_direction: match self.orient {
                        Orient::Horizontal => taffy::FlexDirection::Row,
                        Orient::Vertical => taffy::FlexDirection::Column,
                    },
                    ..Default::default()
                },
                &nodes,
            )
            .unwrap();
        tree.compute_layout(root, taffy::Size {
            width: taffy::AvailableSpace::Definite(self.size.width as _),
            height: taffy::AvailableSpace::Definite(self.size.height as _),
        })
        .unwrap();
        for (id, child) in nodes.iter().zip(&mut self.children) {
            let layout = tree.layout(*id).unwrap();
            child.widget.set_rect(offset(rect_t2e(layout), self.loc));
        }
    }
}

fn rect_t2e(rect: &taffy::Layout) -> Rect {
    Rect::new(
        Point::new(rect.location.x as _, rect.location.y as _),
        Size::new(rect.size.width as _, rect.size.height as _),
    )
}

fn offset(mut a: Rect, offset: Point) -> Rect {
    a.origin += offset.to_vector();
    a
}

impl Layoutable for StackPanel<'_> {
    fn loc(&self) -> Point {
        self.loc
    }

    fn set_loc(&mut self, p: Point) {
        self.loc = p;
        self.render();
    }

    fn size(&self) -> Size {
        self.size
    }

    fn set_size(&mut self, s: Size) {
        self.size = s;
        self.render();
    }

    fn preferred_size(&self) -> Size {
        let mut width = 0.0;
        let mut height = 0.0;
        match self.orient {
            Orient::Horizontal => {
                for child in &self.children {
                    width += child
                        .width
                        .unwrap_or_else(|| child.widget.preferred_size().width);
                    width += child.margin.horizontal();
                    height = child
                        .height
                        .unwrap_or_else(|| child.widget.preferred_size().height)
                        .max(height);
                }
            }
            Orient::Vertical => {
                for child in &self.children {
                    height += child
                        .height
                        .unwrap_or_else(|| child.widget.preferred_size().height);
                    height += child.margin.vertical();
                    width = child
                        .width
                        .unwrap_or_else(|| child.widget.preferred_size().width)
                        .max(width);
                }
            }
        }
        Size::new(width, height)
    }
}
