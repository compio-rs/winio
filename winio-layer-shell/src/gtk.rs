use gtk4::prelude::*;
pub(crate) use gtk4_layer_shell::is_supported;
use gtk4_layer_shell::{
    Edge as GtkEdge, KeyboardMode as GtkKeyboardMode, Layer as GtkLayer, LayerShell as _,
};
use winio::handle::BorrowedWindow;

use crate::{Anchors, Edge, Error, KeyboardInteractivity, Layer, Result};

pub(crate) fn ensure(handle: BorrowedWindow<'_>) -> Result<()> {
    let window = handle.to_gtk();
    if window.is_layer_window() {
        return Ok(());
    }
    if !is_supported() {
        return Err(Error::NotSupported);
    }
    if window.is_realized() {
        return Err(Error::AlreadyShown);
    }

    window.init_layer_shell();
    // Unify close-on-dismiss behavior with the Qt backend (which defaults to
    // closing). gtk4-layer-shell v1.3+ defaults to `respect_close = false`,
    // which would otherwise swallow the `.closed` event and keep the window
    // alive.
    window.set_respect_close(true);

    if window.is_layer_window() {
        Ok(())
    } else {
        Err(Error::NotSupported)
    }
}

pub(crate) fn layer(handle: BorrowedWindow<'_>) -> Result<Layer> {
    let window = handle.to_gtk();
    if !window.is_layer_window() {
        return Err(Error::NotSupported);
    }
    Ok(match window.layer() {
        GtkLayer::Background => Layer::Background,
        GtkLayer::Bottom => Layer::Bottom,
        GtkLayer::Overlay => Layer::Overlay,
        _ => Layer::Top,
    })
}

pub(crate) fn set_layer(handle: BorrowedWindow<'_>, layer: Layer) -> Result<()> {
    ensure(handle)?;
    let layer = match layer {
        Layer::Background => GtkLayer::Background,
        Layer::Bottom => GtkLayer::Bottom,
        Layer::Top => GtkLayer::Top,
        Layer::Overlay => GtkLayer::Overlay,
    };
    handle.to_gtk().set_layer(layer);
    Ok(())
}

pub(crate) fn anchors(handle: BorrowedWindow<'_>) -> Result<Anchors> {
    let window = handle.to_gtk();
    if !window.is_layer_window() {
        return Err(Error::NotSupported);
    }
    let mut anchors = Anchors::empty();
    for (edge, anchor) in [
        (Edge::Top, Anchors::TOP),
        (Edge::Bottom, Anchors::BOTTOM),
        (Edge::Left, Anchors::LEFT),
        (Edge::Right, Anchors::RIGHT),
    ] {
        if window.is_anchor(gtk_edge(edge)) {
            anchors |= anchor;
        }
    }
    Ok(anchors)
}

pub(crate) fn set_anchors(handle: BorrowedWindow<'_>, anchors: Anchors) -> Result<()> {
    ensure(handle)?;
    let window = handle.to_gtk();
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        window.set_anchor(gtk_edge(edge), anchors.contains_edge(edge));
    }
    Ok(())
}

pub(crate) fn margin(handle: BorrowedWindow<'_>, edge: Edge) -> Result<i32> {
    let window = handle.to_gtk();
    if !window.is_layer_window() {
        return Err(Error::NotSupported);
    }
    Ok(window.margin(gtk_edge(edge)))
}

pub(crate) fn set_margin(handle: BorrowedWindow<'_>, edge: Edge, margin: i32) -> Result<()> {
    ensure(handle)?;
    handle.to_gtk().set_margin(gtk_edge(edge), margin);
    Ok(())
}

pub(crate) fn exclusive_zone(handle: BorrowedWindow<'_>) -> Result<i32> {
    let window = handle.to_gtk();
    if !window.is_layer_window() {
        return Err(Error::NotSupported);
    }
    Ok(window.exclusive_zone())
}

pub(crate) fn set_exclusive_zone(handle: BorrowedWindow<'_>, zone: i32) -> Result<()> {
    ensure(handle)?;
    handle.to_gtk().set_exclusive_zone(zone);
    Ok(())
}

pub(crate) fn keyboard_interactivity(handle: BorrowedWindow<'_>) -> Result<KeyboardInteractivity> {
    let window = handle.to_gtk();
    if !window.is_layer_window() {
        return Err(Error::NotSupported);
    }
    Ok(match window.keyboard_mode() {
        GtkKeyboardMode::None => KeyboardInteractivity::None,
        GtkKeyboardMode::Exclusive => KeyboardInteractivity::Exclusive,
        _ => KeyboardInteractivity::OnDemand,
    })
}

pub(crate) fn set_keyboard_interactivity(
    handle: BorrowedWindow<'_>,
    mode: KeyboardInteractivity,
) -> Result<()> {
    ensure(handle)?;
    let mode = match mode {
        KeyboardInteractivity::None => GtkKeyboardMode::None,
        KeyboardInteractivity::OnDemand => GtkKeyboardMode::OnDemand,
        KeyboardInteractivity::Exclusive => GtkKeyboardMode::Exclusive,
    };
    handle.to_gtk().set_keyboard_mode(mode);
    Ok(())
}

pub(crate) fn namespace(handle: BorrowedWindow<'_>) -> Result<String> {
    let window = handle.to_gtk();
    if !window.is_layer_window() {
        return Err(Error::NotSupported);
    }
    Ok(window
        .namespace()
        .map(|s| s.to_string())
        .unwrap_or_default())
}

pub(crate) fn set_namespace(handle: BorrowedWindow<'_>, namespace: &str) -> Result<()> {
    let window = handle.to_gtk();
    // The namespace is passed when the layer surface is created, so it cannot
    // be changed after the window has been realized.
    if window.is_realized() {
        return Err(Error::AlreadyShown);
    }
    ensure(handle)?;
    handle.to_gtk().set_namespace(Some(namespace));
    Ok(())
}

fn gtk_edge(edge: Edge) -> GtkEdge {
    match edge {
        Edge::Top => GtkEdge::Top,
        Edge::Bottom => GtkEdge::Bottom,
        Edge::Left => GtkEdge::Left,
        Edge::Right => GtkEdge::Right,
    }
}
