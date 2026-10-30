use gtk4::prelude::*;
use gtk4_layer_shell::{
    Edge as GtkEdge, KeyboardMode as GtkKeyboardMode, Layer as GtkLayer, LayerShell as _,
    is_supported as gtk_is_supported,
};
use winio::handle::BorrowedWindow;

use crate::{Anchors, Edge, Error, KeyboardInteractivity, Layer, Result};

pub(crate) fn is_supported() -> bool {
    gtk_is_supported()
}

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
    if window.is_layer_window() {
        Ok(())
    } else {
        Err(Error::NotSupported)
    }
}

pub(crate) fn layer(handle: BorrowedWindow<'_>) -> Result<Layer> {
    ensure(handle)?;
    Ok(match handle.to_gtk().layer() {
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
    ensure(handle)?;
    let window = handle.to_gtk();
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
    ensure(handle)?;
    Ok(handle.to_gtk().margin(gtk_edge(edge)))
}

pub(crate) fn set_margin(handle: BorrowedWindow<'_>, edge: Edge, margin: i32) -> Result<()> {
    ensure(handle)?;
    handle.to_gtk().set_margin(gtk_edge(edge), margin);
    Ok(())
}

pub(crate) fn exclusive_zone(handle: BorrowedWindow<'_>) -> Result<i32> {
    ensure(handle)?;
    Ok(handle.to_gtk().exclusive_zone())
}

pub(crate) fn set_exclusive_zone(handle: BorrowedWindow<'_>, zone: i32) -> Result<()> {
    ensure(handle)?;
    handle.to_gtk().set_exclusive_zone(zone);
    Ok(())
}

pub(crate) fn keyboard_interactivity(handle: BorrowedWindow<'_>) -> Result<KeyboardInteractivity> {
    ensure(handle)?;
    Ok(match handle.to_gtk().keyboard_mode() {
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

pub(crate) fn namespace(handle: BorrowedWindow<'_>) -> Result<Option<String>> {
    ensure(handle)?;
    Ok(handle.to_gtk().namespace().map(|s| s.to_string()))
}

pub(crate) fn set_namespace(handle: BorrowedWindow<'_>, namespace: &str) -> Result<()> {
    ensure(handle)?;
    handle.to_gtk().set_namespace(Some(namespace));
    Ok(())
}

pub(crate) fn request_refresh(handle: BorrowedWindow<'_>) -> Result<()> {
    handle.to_gtk().queue_draw();
    Ok(())
}

pub(crate) fn is_configured(handle: BorrowedWindow<'_>) -> bool {
    handle.to_gtk().is_layer_window() && handle.to_gtk().is_realized()
}

pub(crate) fn output_info(_handle: BorrowedWindow<'_>) -> Option<crate::OutputInfo> {
    Some(crate::OutputInfo {
        name: Some("gtk".into()),
        is_primary: true,
    })
}

fn gtk_edge(edge: Edge) -> GtkEdge {
    match edge {
        Edge::Top => GtkEdge::Top,
        Edge::Bottom => GtkEdge::Bottom,
        Edge::Left => GtkEdge::Left,
        Edge::Right => GtkEdge::Right,
    }
}
