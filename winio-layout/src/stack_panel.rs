use taffy::{
    NodeId, Style, TaffyTree,
    prelude::{auto, length, percent},
};
use winio_primitive::{Failable, HAlign, Margin, Orient, Point, Rect, Size, VAlign};

use crate::{LayoutError, Layoutable, layout_child, rect_t2e, render};

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
        widget: &'a mut dyn Layoutable<Error = E>,
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
            let mut preferred_size = child.widget.preferred_size().map_err(LayoutError::Child)?;
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
            let mut min_size = child.widget.min_size().map_err(LayoutError::Child)?;
            min_size.width += child.margin.horizontal();
            min_size.height += child.margin.vertical();
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
}

impl<E> Failable for StackPanel<'_, E> {
    type Error = LayoutError<E>;
}

impl<E> Layoutable for StackPanel<'_, E> {
    fn loc(&self) -> Result<Point, Self::Error> {
        Ok(self.loc)
    }

    fn set_loc(&mut self, p: Point) -> Result<(), Self::Error> {
        self.loc = p;
        self.render()
    }

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(self.size)
    }

    fn set_size(&mut self, s: Size) -> Result<(), Self::Error> {
        self.size = s;
        self.render()
    }

    fn set_rect(&mut self, r: Rect) -> Result<(), Self::Error> {
        self.loc = r.origin;
        self.size = r.size;
        self.render()
    }

    fn preferred_size(&self) -> Result<Size, Self::Error> {
        let (mut tree, root, _) = self.tree()?;
        tree.compute_layout(root, taffy::Size::max_content())?;
        Ok(rect_t2e(tree.layout(root)?, Margin::zero()).size)
    }

    fn min_size(&self) -> Result<Size, Self::Error> {
        let (mut tree, root, _) = self.tree()?;
        tree.compute_layout(root, taffy::Size::min_content())?;
        Ok(rect_t2e(tree.layout(root)?, Margin::zero()).size)
    }
}
