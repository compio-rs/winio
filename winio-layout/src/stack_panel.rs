use taffy::{
    NodeId, Style, TaffyTree,
    prelude::{auto, length, percent},
};
use winio_primitive::{Failable, HAlign, Orient, Point, Rect, Size, VAlign};

use crate::{LayoutChild, LayoutError, layout_child, rect_t2e, render};

layout_child! {
    /// Builder of a child for [`StackPanel`].
    struct StackPanelChild {
        /// Whether to fill the remain space of the container.
        grow: bool = false,
    }
}

/// A stacked layout container.
pub struct StackPanel<'a, E> {
    children: Vec<StackPanelChild<'a, E>>,
    orient: Orient,
    loc: Point,
    size: Size,
}

impl<'a, E> StackPanel<'a, E> {
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
    pub fn push<'b>(
        &'b mut self,
        widget: &'a mut dyn LayoutChild<Error = E>,
    ) -> StackPanelChildBuilder<'a, 'b, E> {
        StackPanelChildBuilder {
            child: StackPanelChild::new(widget),
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
            let min_size = child.child_min_size()?;
            style.min_size = taffy::Size {
                width: length(min_size.width as f32),
                height: length(min_size.height as f32),
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
            let node = tree.new_leaf(style)?;
            nodes.push(node);
        }
        let root = tree.new_with_children(
            Style {
                size: taffy::Size::from_percent(1.0, 1.0),
                flex_direction: match self.orient {
                    Orient::Horizontal => taffy::FlexDirection::Row,
                    Orient::Vertical => taffy::FlexDirection::Column,
                },
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

impl<E> Failable for StackPanel<'_, E> {
    type Error = E;
}

impl<E> LayoutChild for StackPanel<'_, E> {
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
