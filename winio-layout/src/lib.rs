//! Layout primitives and containers.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![warn(missing_docs)]

#[doc(hidden)]
pub use paste::paste as __paste;
use taffy::{Layout, NodeId, TaffyTree};
#[doc(hidden)]
pub use winio_primitive::{HAlign, VAlign};
use winio_primitive::{Margin, Point, Rect, Size};

/// Trait for a widget to set visibility.
pub trait Visible {
    /// If the widget is visible.
    fn is_visible(&self) -> bool;

    /// Set the visibility.
    fn set_visible(&mut self, v: bool);

    /// Show the widget.
    fn show(&mut self) {
        self.set_visible(true);
    }

    /// Hide the widget.
    fn hide(&mut self) {
        self.set_visible(false);
    }
}

/// Trait for a widget to enable or disable.
pub trait Enable {
    /// If the widget is enabled.
    fn is_enabled(&self) -> bool;

    /// Set if the widget is enabled.
    fn set_enabled(&mut self, v: bool);

    /// Enable the widget.
    fn enable(&mut self) {
        self.set_enabled(true);
    }

    /// Disable the widget.
    fn disable(&mut self) {
        self.set_enabled(false);
    }
}

/// Trait for a layoutable widget.
///
/// To create a responsive layout, always set location and size together.
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

    /// Min acceptable size.
    fn min_size(&self) -> Size {
        self.preferred_size()
    }
}

trait LayoutChild {
    fn margin(&self) -> Margin;

    fn set_rect(&mut self, r: Rect);

    fn layout(&mut self, layout: &Layout, loc: Point) {
        self.set_rect(offset(rect_t2e(layout, self.margin()), loc))
    }
}

mod grid;
pub use grid::*;

mod stack_panel;
pub use stack_panel::*;

#[cfg(test)]
mod test;

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

macro_rules! __layout_child {
    ($(#[$sm:meta])* struct $name:ident { $($(#[$m:meta])* $f:ident: $t:ty = $e:expr_2021),*$(,)? }) => {
        struct $name<'a> {
            widget: &'a mut dyn $crate::Layoutable,
            width: Option<f64>,
            height: Option<f64>,
            margin: $crate::Margin,
            halign: $crate::HAlign,
            valign: $crate::VAlign,
            $(
                $(#[$m])*
                $f: $t,
            )*
        }
        impl<'a> $name<'a> {
            #[allow(unused_doc_comments)]
            pub fn new(widget: &'a mut dyn $crate::Layoutable) -> Self {
                Self {
                    widget,
                    width: None,
                    height: None,
                    margin: $crate::Margin::zero(),
                    halign: $crate::HAlign::Stretch,
                    valign: $crate::VAlign::Stretch,
                    $(
                        $(#[$m])*
                        $f: $e,
                    )*
                }
            }
        }
        impl $crate::LayoutChild for $name<'_> {
            fn margin(&self) -> $crate::Margin {
                self.margin
            }

            fn set_rect(&mut self, r: Rect) {
                self.widget.set_rect(r)
            }
        }
        $crate::__paste! {
            $(#[$sm])*
            pub struct [<$name Builder>]<'a, 'b> {
                child: $name<'a>,
                children: &'b mut Vec<$name<'a>>,
            }
            impl [<$name Builder>]<'_, '_> {
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
                pub fn size(self, s: $crate::Size) -> Self {
                    self.width(s.width).height(s.height)
                }

                /// Margin of the child.
                pub fn margin(mut self, m: $crate::Margin) -> Self {
                    self.child.margin = m;
                    self
                }

                /// Horizontal alignment in the available area.
                pub fn halign(mut self, v: $crate::HAlign) -> Self {
                    self.child.halign = v;
                    self
                }

                /// Vertical alignment in the available area.
                pub fn valign(mut self, v: $crate::VAlign) -> Self {
                    self.child.valign = v;
                    self
                }

                $(
                    $(#[$m])*
                    pub fn $f(mut self, v: $t) -> Self {
                        self.child.$f = v;
                        self
                    }
                )*

                /// Add the child to the container.
                pub fn finish(self) {
                    self.children.push(self.child);
                }
            }
        }
    };
}
pub(crate) use __layout_child as layout_child;

fn render(
    mut tree: TaffyTree,
    root: NodeId,
    nodes: Vec<NodeId>,
    loc: Point,
    size: Size,
    children: &mut [impl LayoutChild],
) {
    tree.compute_layout(
        root,
        taffy::Size {
            width: taffy::AvailableSpace::Definite(size.width as _),
            height: taffy::AvailableSpace::Definite(size.height as _),
        },
    )
    .unwrap();
    for (id, child) in nodes.iter().zip(children) {
        let layout = tree.layout(*id).unwrap();
        child.layout(layout, loc);
    }
}

/// Helper macro for layouts in `Component::render`.
///
/// ```ignore
/// # use winio::prelude::*;
/// # struct MainModel {
/// #     window: Child<Window>,
/// #     canvas: Child<Canvas>,
/// # }
/// # impl MainModel { fn foo(&mut self) {
/// let csize = self.window.client_size();
/// {
///     let mut grid = layout! {
///         Grid::from_str("1*,2*,1*", "1*,2*,1*").unwrap(),
///         self.canvas => { column: 1, row: 1 },
///     };
///     grid.set_size(csize);
/// }
/// # } }
/// ```
#[macro_export]
macro_rules! layout {
    ($root:expr_2021, $($e:expr_2021 $(=>  { $($t:tt)* })?),+$(,)?) => {{
        #[allow(unused_mut)]
        let mut root = $root;
        $(
            $crate::__layout_push!(root, &mut $e, $($($t)*)?);
        )+
        root
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __layout_push {
    ($root:expr_2021, $e:expr_2021,) => {
        $root.push($e).finish();
    };
    ($root:expr_2021, $e:expr_2021, $($(#[$me:meta])* $p:ident : $v:expr_2021),+$(,)?) => {
        let builder = $root.push($e);
        $(
            $(#[$me])*
            let builder = builder.$p($v);
        )+
        builder.finish();
    };
}
