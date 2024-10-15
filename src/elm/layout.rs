use std::{fmt::Display, num::ParseFloatError, str::FromStr};

use taffy::{
    NodeId, Style, TaffyTree, TrackSizingFunction,
    prelude::{auto, fr, length, line, percent, span},
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

fn rect_t2e(rect: &taffy::Layout, margin: Margin) -> Rect {
    Rect::new(
        Point::new(
            rect.location.x as f64 + margin.left,
            rect.location.y as f64 + margin.top,
        ),
        Size::new(
            rect.size.width as f64 - margin.horizontal(),
            rect.size.height as f64 - margin.vertical(),
        ),
    )
}

fn offset(mut a: Rect, offset: Point) -> Rect {
    a.origin += offset.to_vector();
    a
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
    column_span: usize,
    row_span: usize,
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
            column_span: 1,
            row_span: 1,
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
    pub fn column_span(mut self, s: usize) -> Self {
        self.child.column_span = s;
        self
    }

    /// The row span in the grid.
    pub fn row_span(mut self, s: usize) -> Self {
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

    fn tree(&self) -> (TaffyTree, NodeId, Vec<NodeId>) {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        let mut nodes = vec![];
        for child in &self.children {
            let mut preferred_size = child.widget.preferred_size();
            preferred_size.width += child.margin.horizontal();
            preferred_size.height += child.margin.vertical();
            let mut style = Style::default();
            style.size.width = match child.width {
                Some(w) => length(w as f32),
                None => match (self.orient, child.halign, child.grow) {
                    (Orient::Vertical, HAlign::Stretch, _) => percent(1.0),
                    (Orient::Horizontal, _, true) => auto(),
                    _ => length(preferred_size.width as f32),
                },
            };
            style.size.height = match child.height {
                Some(h) => length(h as f32),
                None => match (self.orient, child.valign, child.grow) {
                    (Orient::Horizontal, VAlign::Stretch, _) => percent(1.0),
                    (Orient::Vertical, _, true) => auto(),
                    _ => length(preferred_size.height as f32),
                },
            };
            style.min_size = taffy::Size {
                width: length(preferred_size.width as f32),
                height: length(preferred_size.height as f32),
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
        (tree, root, nodes)
    }

    fn render(&mut self) {
        let (mut tree, root, nodes) = self.tree();
        tree.compute_layout(root, taffy::Size {
            width: taffy::AvailableSpace::Definite(self.size.width as _),
            height: taffy::AvailableSpace::Definite(self.size.height as _),
        })
        .unwrap();
        for (id, child) in nodes.iter().zip(&mut self.children) {
            let layout = tree.layout(*id).unwrap();
            child
                .widget
                .set_rect(offset(rect_t2e(layout, child.margin), self.loc));
        }
    }
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

    fn set_rect(&mut self, r: Rect) {
        self.loc = r.origin;
        self.size = r.size;
        self.render();
    }

    fn preferred_size(&self) -> Size {
        let (mut tree, root, _) = self.tree();
        tree.compute_layout(root, taffy::Size::max_content())
            .unwrap();
        rect_t2e(tree.layout(root).unwrap(), Margin::zero()).size
    }
}

/// Error can be returned when parsing [`GridLength`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ParseGridLengthError {
    /// Invalid length value.
    InvalidLength(ParseFloatError),
}

impl Display for ParseGridLengthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength(e) => write!(f, "invalid length value: {}", e),
        }
    }
}

impl std::error::Error for ParseGridLengthError {}

/// The width or height of a grid cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridLength {
    /// The length is determined automatically.
    Auto,
    /// Represents a relative ratio.
    Stretch(f64),
    /// Fixed length.
    Length(f64),
}

impl FromStr for GridLength {
    type Err = ParseGridLengthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("auto") {
            Ok(Self::Auto)
        } else if let Some(s) = s.strip_suffix('*') {
            s.parse::<f64>()
                .map(Self::Stretch)
                .map_err(ParseGridLengthError::InvalidLength)
        } else {
            s.parse::<f64>()
                .map(Self::Length)
                .map_err(ParseGridLengthError::InvalidLength)
        }
    }
}

impl From<GridLength> for TrackSizingFunction {
    fn from(value: GridLength) -> Self {
        match value {
            GridLength::Auto => auto(),
            GridLength::Length(v) => length(v as f32),
            GridLength::Stretch(v) => fr(v as f32),
        }
    }
}

impl From<&GridLength> for TrackSizingFunction {
    fn from(value: &GridLength) -> Self {
        TrackSizingFunction::from(*value)
    }
}

/// A grid layout container.
pub struct Grid<'a> {
    children: Vec<LayoutChild<'a>>,
    columns: Vec<GridLength>,
    rows: Vec<GridLength>,
    loc: Point,
    size: Size,
}

impl<'a> Grid<'a> {
    /// Create [`Grid`].
    pub fn new(columns: Vec<GridLength>, rows: Vec<GridLength>) -> Self {
        Self {
            children: vec![],
            columns,
            rows,
            loc: Point::zero(),
            size: Size::zero(),
        }
    }

    /// Create [`Grid`] with string-representative of grid lengths.
    pub fn from_str(
        columns: impl AsRef<str>,
        rows: impl AsRef<str>,
    ) -> Result<Self, ParseGridLengthError> {
        Ok(Self::new(
            Self::parse_grid_lengths(columns.as_ref())?,
            Self::parse_grid_lengths(rows.as_ref())?,
        ))
    }

    fn parse_grid_lengths(s: &str) -> Result<Vec<GridLength>, ParseGridLengthError> {
        let mut lengths = vec![];
        for s in s.split(',') {
            let s = s.trim();
            lengths.push(s.parse()?);
        }
        Ok(lengths)
    }

    /// Push a child into the panel.
    pub fn push<'b>(&'b mut self, widget: &'a mut dyn Layoutable) -> LayoutChildBuilder<'a, 'b> {
        LayoutChildBuilder {
            child: LayoutChild::new(widget),
            children: &mut self.children,
        }
    }

    fn tree(&self) -> (TaffyTree, NodeId, Vec<NodeId>) {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        let mut nodes = vec![];
        for child in &self.children {
            let mut preferred_size = child.widget.preferred_size();
            preferred_size.width += child.margin.horizontal();
            preferred_size.height += child.margin.vertical();
            let mut style = Style::default();
            style.size.width = match child.width {
                Some(w) => length(w as f32),
                None => match child.halign {
                    HAlign::Stretch => auto(),
                    _ => length(preferred_size.width as f32),
                },
            };
            style.size.height = match child.height {
                Some(h) => length(h as f32),
                None => match child.valign {
                    VAlign::Stretch => auto(),
                    _ => length(preferred_size.height as f32),
                },
            };
            style.min_size = taffy::Size {
                width: length(preferred_size.width as f32),
                height: length(preferred_size.height as f32),
            };

            if matches!(child.valign, VAlign::Top | VAlign::Center) {
                style.margin.bottom = auto();
            }
            if matches!(child.valign, VAlign::Bottom | VAlign::Center) {
                style.margin.top = auto();
            }
            if matches!(child.halign, HAlign::Left | HAlign::Center) {
                style.margin.right = auto();
            }
            if matches!(child.halign, HAlign::Right | HAlign::Center) {
                style.margin.left = auto();
            }

            style.grid_column.start = line(child.column as i16 + 1);
            style.grid_row.start = line(child.row as i16 + 1);

            let cspan = child.column_span;
            if cspan > 1 {
                style.grid_column.end = span(cspan as u16);
            }

            let rspan = child.row_span;
            if rspan > 1 {
                style.grid_row.end = span(rspan as u16);
            }

            let node = tree.new_leaf(style).unwrap();
            nodes.push(node);
        }
        let root = tree
            .new_with_children(
                Style {
                    display: taffy::Display::Grid,
                    size: taffy::Size::from_percent(1.0, 1.0),
                    grid_template_columns: self
                        .columns
                        .iter()
                        .map(TrackSizingFunction::from)
                        .collect(),
                    grid_template_rows: self.rows.iter().map(TrackSizingFunction::from).collect(),
                    ..Default::default()
                },
                &nodes,
            )
            .unwrap();
        (tree, root, nodes)
    }

    fn render(&mut self) {
        let (mut tree, root, nodes) = self.tree();
        tree.compute_layout(root, taffy::Size {
            width: taffy::AvailableSpace::Definite(self.size.width as _),
            height: taffy::AvailableSpace::Definite(self.size.height as _),
        })
        .unwrap();
        for (id, child) in nodes.iter().zip(&mut self.children) {
            let layout = tree.layout(*id).unwrap();
            child
                .widget
                .set_rect(offset(rect_t2e(layout, child.margin), self.loc));
        }
    }
}

impl Layoutable for Grid<'_> {
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

    fn set_rect(&mut self, r: Rect) {
        self.loc = r.origin;
        self.size = r.size;
        self.render();
    }

    fn preferred_size(&self) -> Size {
        let (mut tree, root, _) = self.tree();
        tree.compute_layout(root, taffy::Size::max_content())
            .unwrap();
        rect_t2e(tree.layout(root).unwrap(), Margin::zero()).size
    }
}
