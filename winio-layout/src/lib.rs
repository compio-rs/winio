//! Layout primitives and containers.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

#[doc(hidden)]
pub use paste::paste as __paste;
pub use taffy::TaffyError;
use taffy::{Layout, NodeId, TaffyTree};
use thiserror::Error;
#[doc(hidden)]
pub use winio_primitive::{Failable, HAlign, Layoutable, VAlign};
use winio_primitive::{Margin, Point, Rect, Size};

trait LayoutChild: Failable {
    fn margin(&self) -> Margin;

    fn set_rect(&mut self, r: Rect) -> Result<(), Self::Error>;

    fn layout(&mut self, layout: &Layout, loc: Point) -> Result<(), Self::Error> {
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
    ($(#[$sm:meta])* struct $name:ident { $($(#[$m:meta])* $f:ident: $t:ty = $e:expr),*$(,)? }) => {
        struct $name<'a, E> {
            widget: &'a mut dyn $crate::Layoutable<Error = E>,
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
        impl<'a, E> $name<'a, E> {
            #[allow(unused_doc_comments)]
            pub fn new(widget: &'a mut dyn $crate::Layoutable<Error = E>) -> Self {
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
        impl<E> $crate::Failable for $name<'_, E> {
            type Error = E;
        }
        impl<E> $crate::LayoutChild for $name<'_, E> {
            fn margin(&self) -> $crate::Margin {
                self.margin
            }

            fn set_rect(&mut self, r: Rect) -> Result<(), Self::Error> {
                self.widget.set_rect(r)
            }
        }
        $crate::__paste! {
            $(#[$sm])*
            pub struct [<$name Builder>]<'a, 'b, E> {
                child: $name<'a, E>,
                children: &'b mut Vec<$name<'a, E>>,
            }
            impl<E> [<$name Builder>]<'_, '_, E> {
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

/// Errors that can occur during layout.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum LayoutError<E> {
    /// Taffy layout error.
    #[error("Taffy layout error: {0}")]
    Taffy(#[from] TaffyError),
    /// Child layout error.
    #[error("Child layout error: {0}")]
    Child(E),
}

fn render<E>(
    mut tree: TaffyTree,
    root: NodeId,
    nodes: Vec<NodeId>,
    loc: Point,
    size: Size,
    children: &mut [impl LayoutChild<Error = E>],
) -> Result<(), LayoutError<E>> {
    tree.compute_layout(
        root,
        taffy::Size {
            width: taffy::AvailableSpace::Definite(size.width as _),
            height: taffy::AvailableSpace::Definite(size.height as _),
        },
    )?;
    for (id, child) in nodes.iter().zip(children) {
        let layout = tree.layout(*id)?;
        child.layout(layout, loc).map_err(LayoutError::Child)?;
    }
    Ok(())
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
    ($root:expr, $($e:expr $(=>  { $($t:tt)* })?),+$(,)?) => {{
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
    ($root:expr, $e:expr,) => {
        $root.push($e).finish();
    };
    ($root:expr, $e:expr, $($(#[$me:meta])* $p:ident : $v:expr),+$(,)?) => {
        let builder = $root.push($e);
        $(
            $(#[$me])*
            let builder = builder.$p($v);
        )+
        builder.finish();
    };
}
