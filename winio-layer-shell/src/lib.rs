//! Wayland layer shell for [`winio`].
//!
//! Extension trait [`LayerShellExt`] configures native windows (Qt via
//! `layer-shell-qt`, GTK via `gtk4-layer-shell`) as layer surfaces.
//!
//! Configure before `show()`; surfaces are fixed at creation time.
//!
//! Note (Qt Wayland): `QWindow` needs non-zero width (`Size::new(0.0, ...)`
//! sets `WA_OutsideWSRange`, skipping `initWindow()` — layer never registers).
//!
//! # Features
//!
//! Select the feature matching the winio backend:
//!
//! * `gtk`: use [`gtk4-layer-shell`]. Requires the `gtk4-layer-shell`
//!   development package.
//! * `qt`: use [`layer-shell-qt`]. Requires the `layer-shell-qt` development
//!   package. Set `LAYERSHELLQT_INCLUDE_DIR` and `LAYERSHELLQT_LIB_DIR` when
//!   the headers and libraries are not in the Qt installation prefix.
//!
//! The features are mutually exclusive, just like the winio backends.
//!
//! # Usage
//!
//! Configure the window before showing it. Layer shell is applied when the
//! native surface is created, and a window that is already visible cannot be
//! converted into a layer surface.
//!
//! ## Platform notes
//!
//! * **Qt (Wayland)**: the native `QWindow` must have a non-zero width. A
//!   zero-width widget (`Size::new(0.0, ...)` on Qt) causes QtWidgets to set
//!   the `WA_OutsideWSRange` attribute, which skips
//!   `QWaylandWindow::setVisible()` / `initWindow()` — so the layer surface
//!   never registers with the compositor. Use `Size::new(non_zero, ...)` (e.g.
//!   the full screen width for a bar) and rely on the compositor's
//!   `exclusive_zone` for positioning.
//!
//! ```ignore
//! use winio::prelude::*;
//! use winio_layer_shell::{Anchors, Layer, LayerShellExt};
//!
//! // `window` is a `Child<Window>` or any other type implementing
//! // `winio::handle::AsWindow`.
//! window.init_layer_shell()?;
//! window.set_layer(Layer::Top)?;
//! window.set_anchors(Anchors::TOP | Anchors::LEFT | Anchors::RIGHT)?;
//! window.set_exclusive_zone(30)?;
//! window.set_namespace("my-panel")?;
//! window.show()?;
//! ```
//!
//! [`gtk4-layer-shell`]: https://github.com/pentamassiv/gtk4-layer-shell-gir
//! [`layer-shell-qt`]: https://github.com/KDE/layer-shell-qt

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

#[cfg(all(feature = "gtk", feature = "qt"))]
compile_error!("You must choose only one of these features: [\"gtk\", \"qt\"]");

use bitflags::bitflags;
use winio::handle::AsWindow;

cfg_if::cfg_if! {
    if #[cfg(all(
        feature = "gtk",
        not(any(windows, target_vendor = "apple", target_os = "android"))
    ))] {
        mod gtk;
        use gtk as imp;
    } else if #[cfg(all(
        feature = "qt",
        not(any(windows, target_vendor = "apple", target_os = "android"))
    ))] {
        mod qt;
        use qt as imp;
    } else {
        mod stub;
        use stub as imp;
    }
}

/// An edge of a layer surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    /// The top edge.
    Top,
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
    /// The right edge.
    Right,
}

/// Keyboard focus policy of a layer surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyboardInteractivity {
    /// The surface never receives keyboard focus.
    None,
    /// The surface receives keyboard focus when the user requests it, for
    /// example by clicking on it.
    OnDemand,
    /// The surface receives keyboard focus exclusively while it is mapped.
    Exclusive,
}

/// The layer where a surface is placed.
///
/// Layers are ordered from the bottom of the screen to the top. The compositor
/// may place other surfaces, such as normal windows, between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    /// The background layer, below all other surfaces.
    Background,
    /// The bottom layer, above the background but below normal windows.
    Bottom,
    /// The top layer, above normal windows.
    Top,
    /// The overlay layer, above all other surfaces.
    Overlay,
}

bitflags! {
    /// A set of edges a layer surface is anchored to.
    ///
    /// Anchoring to two opposite edges stretches the surface in that
    /// direction. Anchoring to no edge lets the compositor place the surface.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Anchors: u8 {
        /// Anchor to the top edge.
        const TOP = 1 << 0;
        /// Anchor to the bottom edge.
        const BOTTOM = 1 << 1;
        /// Anchor to the left edge.
        const LEFT = 1 << 2;
        /// Anchor to the right edge.
        const RIGHT = 1 << 3;
    }
}

impl Anchors {
    /// Whether the given edge is anchored.
    pub fn contains_edge(self, edge: Edge) -> bool {
        self.contains(match edge {
            Edge::Top => Self::TOP,
            Edge::Bottom => Self::BOTTOM,
            Edge::Left => Self::LEFT,
            Edge::Right => Self::RIGHT,
        })
    }
}

/// A refresh/repaint request for a layer surface.
///
/// Layer surfaces receive `Configure` events from the compositor that specify
/// their new size. After applying the size change, the surface must submit a
/// new buffer (present). This enum controls when that happens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RefreshRequest {
    /// No refresh is needed; the event loop can sleep.
    #[default]
    Wait,
    /// Redraw at the next available frame (next `present` slot).
    NextFrame,
}

/// Multi-output information for a layer surface.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputInfo {
    /// The output name or identifier.
    pub name: Option<String>,
    /// Whether this is the primary output.
    pub is_primary: bool,
}

/// Errors returned by layer shell operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Layer shell is not available on this platform, backend, or compositor.
    #[error("layer shell is not supported")]
    NotSupported,
    /// The native window is not available yet.
    #[error("the native window is not available")]
    NoWindow,
    /// The window is already visible and cannot become a layer surface.
    #[error("layer shell must be configured before the window is shown")]
    AlreadyShown,
}

/// Result type for layer shell operations.
pub type Result<T> = std::result::Result<T, Error>;

mod sealed {
    use winio::handle::AsWindow;

    pub trait Sealed {}

    impl<T: AsWindow + ?Sized> Sealed for T {}
}

/// Extension trait for configuring a window as a Wayland layer surface.
///
/// The trait is implemented for every type that implements
/// [`winio::handle::AsWindow`], including [`winio::widgets::Window`] and
/// [`winio::handle::BorrowedWindow`].
///
/// Call any of the setters, or [`init_layer_shell`](Self::init_layer_shell),
/// before showing the window. Calling a setter initializes the window
/// automatically.
pub trait LayerShellExt: sealed::Sealed + AsWindow {
    /// Whether layer shell is supported by the current platform and
    /// compositor.
    ///
    /// This may perform a Wayland roundtrip on the first call.
    fn is_layer_shell_supported(&self) -> bool;

    /// Prepare the window to become a layer surface.
    ///
    /// This must be called before the window is shown. It is idempotent, and
    /// the setters call it automatically, so calling it explicitly is
    /// optional.
    fn init_layer_shell(&self) -> Result<()>;

    /// Get the layer of the surface.
    fn layer(&self) -> Result<Layer>;

    /// Set the layer of the surface.
    ///
    /// The default is [`Layer::Top`].
    fn set_layer(&mut self, layer: Layer) -> Result<()>;

    /// Get the anchored edges of the surface.
    fn anchors(&self) -> Result<Anchors>;

    /// Set the anchored edges of the surface.
    ///
    /// The default is [`Anchors::empty`].
    fn set_anchors(&mut self, anchors: Anchors) -> Result<()>;

    /// Get the margin of an edge in logical pixels.
    fn margin(&self, edge: Edge) -> Result<i32>;

    /// Set the margin of an edge in logical pixels.
    ///
    /// The margin only has an effect on an anchored edge. The default is `0`.
    fn set_margin(&mut self, edge: Edge, margin: i32) -> Result<()>;

    /// Get the exclusive zone in logical pixels.
    fn exclusive_zone(&self) -> Result<i32>;

    /// Set the exclusive zone in logical pixels.
    ///
    /// The exclusive zone is the space reserved for the surface at its
    /// anchored edge. Use a negative value to let the compositor calculate it
    /// automatically. The default is `0`.
    fn set_exclusive_zone(&mut self, zone: i32) -> Result<()>;

    /// Get the keyboard focus policy.
    fn keyboard_interactivity(&self) -> Result<KeyboardInteractivity>;

    /// Set the keyboard focus policy.
    ///
    /// The default is [`KeyboardInteractivity::None`].
    fn set_keyboard_interactivity(&mut self, mode: KeyboardInteractivity) -> Result<()>;

    /// Get the namespace of the surface.
    fn namespace(&self) -> Result<Option<String>>;

    /// Set the namespace of the surface.
    ///
    /// The namespace identifies the surface to the compositor. The default is
    /// backend specific.
    fn set_namespace(&mut self, namespace: &str) -> Result<()>;

    /// Request a refresh/repaint of the surface.
    ///
    /// Call after responding to a compositor `Configure` event or when
    /// content changed. The native backend submits a new buffer.
    fn request_refresh(&self) -> Result<()>;

    /// Get basic multi-output info for the surface.
    ///
    /// On Qt this reads the native `QWindow`'s screen; on GTK it reads the
    /// `GdkMonitor`; on stub/non-Wayland it returns empty info.
    fn output_info(&self) -> Option<OutputInfo>;

    /// Whether the native surface has been configured by the compositor.
    ///
    /// This is set to `true` once the layer-shell surface receives its
    /// initial `Configure` event (on Qt: when
    /// `LayerShellQt::Window::eventFilter`
    /// processes `PlatformSurface::SurfaceCreated`; on GTK: when the
    /// `gtk4-layer-shell` configure signal fires).
    fn is_configured(&self) -> bool;
}

impl<T: AsWindow + ?Sized> LayerShellExt for T {
    fn is_layer_shell_supported(&self) -> bool {
        imp::is_supported()
    }

    fn init_layer_shell(&self) -> Result<()> {
        imp::ensure(self.as_window())
    }

    fn layer(&self) -> Result<Layer> {
        imp::layer(self.as_window())
    }

    fn set_layer(&mut self, layer: Layer) -> Result<()> {
        imp::set_layer(self.as_window(), layer)
    }

    fn anchors(&self) -> Result<Anchors> {
        imp::anchors(self.as_window())
    }

    fn set_anchors(&mut self, anchors: Anchors) -> Result<()> {
        imp::set_anchors(self.as_window(), anchors)
    }

    fn margin(&self, edge: Edge) -> Result<i32> {
        imp::margin(self.as_window(), edge)
    }

    fn set_margin(&mut self, edge: Edge, margin: i32) -> Result<()> {
        imp::set_margin(self.as_window(), edge, margin)
    }

    fn exclusive_zone(&self) -> Result<i32> {
        imp::exclusive_zone(self.as_window())
    }

    fn set_exclusive_zone(&mut self, zone: i32) -> Result<()> {
        imp::set_exclusive_zone(self.as_window(), zone)
    }

    fn keyboard_interactivity(&self) -> Result<KeyboardInteractivity> {
        imp::keyboard_interactivity(self.as_window())
    }

    fn set_keyboard_interactivity(&mut self, mode: KeyboardInteractivity) -> Result<()> {
        imp::set_keyboard_interactivity(self.as_window(), mode)
    }

    fn namespace(&self) -> Result<Option<String>> {
        imp::namespace(self.as_window())
    }

    fn set_namespace(&mut self, namespace: &str) -> Result<()> {
        imp::set_namespace(self.as_window(), namespace)
    }

    fn request_refresh(&self) -> Result<()> {
        imp::request_refresh(self.as_window())
    }

    fn output_info(&self) -> Option<OutputInfo> {
        imp::output_info(self.as_window())
    }

    fn is_configured(&self) -> bool {
        imp::is_configured(self.as_window())
    }
}
