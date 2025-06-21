use taffy::{
    NodeId, Style, TaffyTree,
    prelude::{auto, length, percent},
};

use super::{layout_child, rect_t2e, render};
use crate::{HAlign, Layoutable, Margin, Orient, Point, Rect, Size, VAlign};

layout_child! {
    /// Builder of a child for [`StackPanel`].
    struct StackPanelChild {
        /// Whether to fill the remain space of the container.
        grow: bool = false,
    }
}

/// A stacked layout container.
pub struct StackPanel<'a> {
    children: Vec<StackPanelChild<'a>>,
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
    pub fn push<'b>(
        &'b mut self,
        widget: &'a mut dyn Layoutable,
    ) -> StackPanelChildBuilder<'a, 'b> {
        StackPanelChildBuilder {
            child: StackPanelChild::new(widget),
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
        let (tree, root, nodes) = self.tree();
        render(tree, root, nodes, self.loc, self.size, &mut self.children)
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
