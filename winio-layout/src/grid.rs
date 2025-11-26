use std::{fmt::Display, num::ParseFloatError, str::FromStr};

use taffy::{
    GridTemplateComponent, NodeId, Style, TaffyTree, TrackSizingFunction,
    prelude::{auto, fr, length, line, span},
};
use winio_primitive::Failable;

use super::{layout_child, rect_t2e, render};
use crate::{HAlign, LayoutChild, LayoutError, Point, Rect, Size, VAlign};

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
            Self::InvalidLength(e) => write!(f, "invalid length value: {e}"),
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

layout_child! {
    /// Builder of a child for [`Grid`].
    struct GridChild {
        /// The column index in the grid.
        column: usize = 0,
        /// The row index in the grid.
        row: usize = 0,
        /// The column span in the grid.
        column_span: usize = 1,
        /// The row span in the grid.
        row_span: usize = 1,
    }
}

/// A grid layout container.
pub struct Grid<'a, E> {
    children: Vec<GridChild<'a, E>>,
    columns: Vec<GridLength>,
    rows: Vec<GridLength>,
    loc: Point,
    size: Size,
}

impl<'a, E> Grid<'a, E> {
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
    pub fn push<'b>(
        &'b mut self,
        widget: &'a mut dyn LayoutChild<Error = E>,
    ) -> GridChildBuilder<'a, 'b, E> {
        GridChildBuilder {
            child: GridChild::new(widget),
            children: &mut self.children,
        }
    }

    fn tree(&self) -> Result<(TaffyTree, NodeId, Vec<NodeId>), LayoutError<E>> {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        let mut nodes = vec![];
        for child in &self.children {
            let preferred_size = child.child_preferred_size()?;
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
            let min_size = child.child_min_size()?;
            style.min_size = taffy::Size {
                width: length(min_size.width as f32),
                height: length(min_size.height as f32),
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

            let node = tree.new_leaf(style)?;
            nodes.push(node);
        }
        let root = tree.new_with_children(
            Style {
                display: taffy::Display::Grid,
                size: taffy::Size::from_percent(1.0, 1.0),
                grid_template_columns: self
                    .columns
                    .iter()
                    .map(TrackSizingFunction::from)
                    .map(GridTemplateComponent::Single)
                    .collect(),
                grid_template_rows: self
                    .rows
                    .iter()
                    .map(TrackSizingFunction::from)
                    .map(GridTemplateComponent::Single)
                    .collect(),
                ..Default::default()
            },
            &nodes,
        )?;
        Ok((tree, root, nodes))
    }

    fn render(&mut self) -> Result<(), LayoutError<E>> {
        let (tree, root, nodes) = self.tree()?;
        render(tree, root, nodes, self.loc, self.size, &mut self.children)
    }

    /// Move the location.
    pub fn set_loc(&mut self, p: Point) -> Result<(), LayoutError<E>> {
        LayoutChild::set_child_loc(self, p)
    }

    /// Resize.
    pub fn set_size(&mut self, s: Size) -> Result<(), LayoutError<E>> {
        LayoutChild::set_child_size(self, s)
    }

    /// Set the location and size.
    pub fn set_rect(&mut self, r: Rect) -> Result<(), LayoutError<E>> {
        LayoutChild::set_child_rect(self, r)
    }
}

impl<E> Failable for Grid<'_, E> {
    type Error = E;
}

impl<E> LayoutChild for Grid<'_, E> {
    fn set_child_loc(&mut self, p: Point) -> Result<(), LayoutError<Self::Error>> {
        self.loc = p;
        self.render()
    }

    fn set_child_size(&mut self, s: Size) -> Result<(), LayoutError<Self::Error>> {
        self.size = s;
        self.render()
    }

    fn set_child_rect(&mut self, r: Rect) -> Result<(), LayoutError<Self::Error>> {
        self.loc = r.origin;
        self.size = r.size;
        self.render()
    }

    fn child_preferred_size(&self) -> Result<Size, LayoutError<Self::Error>> {
        let (mut tree, root, _) = self.tree()?;
        tree.compute_layout(root, taffy::Size::max_content())?;
        Ok(rect_t2e(tree.layout(root)?).size)
    }

    fn child_min_size(&self) -> Result<Size, LayoutError<Self::Error>> {
        let (mut tree, root, _) = self.tree()?;
        tree.compute_layout(root, taffy::Size::min_content())?;
        Ok(rect_t2e(tree.layout(root)?).size)
    }
}
