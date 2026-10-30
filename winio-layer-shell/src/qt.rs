use winio::handle::BorrowedWindow;

use crate::{Anchors, Edge, Error, KeyboardInteractivity, Layer, Result};

fn widget(handle: BorrowedWindow<'_>) -> *mut ffi::QWidget {
    handle.as_qt()
}

pub(crate) fn is_supported() -> bool {
    ffi::winio_layer_shell_is_supported()
}

pub(crate) fn ensure(handle: BorrowedWindow<'_>) -> Result<()> {
    match unsafe { ffi::winio_layer_shell_init(widget(handle)) } {
        0 => Ok(()),
        1 => Err(Error::NotSupported),
        3 => Err(Error::AlreadyShown),
        _ => Err(Error::NoWindow),
    }
}

pub(crate) fn layer(handle: BorrowedWindow<'_>) -> Result<Layer> {
    ensure(handle)?;
    Ok(
        match unsafe { ffi::winio_layer_shell_layer(widget(handle)) } {
            0 => Layer::Background,
            1 => Layer::Bottom,
            3 => Layer::Overlay,
            _ => Layer::Top,
        },
    )
}

pub(crate) fn set_layer(handle: BorrowedWindow<'_>, layer: Layer) -> Result<()> {
    ensure(handle)?;
    let layer = match layer {
        Layer::Background => 0,
        Layer::Bottom => 1,
        Layer::Top => 2,
        Layer::Overlay => 3,
    };
    unsafe { ffi::winio_layer_shell_set_layer(widget(handle), layer) };
    Ok(())
}

pub(crate) fn anchors(handle: BorrowedWindow<'_>) -> Result<Anchors> {
    ensure(handle)?;
    Ok(Anchors::from_bits_truncate(
        unsafe { ffi::winio_layer_shell_anchors(widget(handle)) } as u8,
    ))
}

pub(crate) fn set_anchors(handle: BorrowedWindow<'_>, anchors: Anchors) -> Result<()> {
    ensure(handle)?;
    unsafe { ffi::winio_layer_shell_set_anchors(widget(handle), anchors.bits().into()) };
    Ok(())
}

pub(crate) fn margin(handle: BorrowedWindow<'_>, edge: Edge) -> Result<i32> {
    ensure(handle)?;
    Ok(unsafe { ffi::winio_layer_shell_margin(widget(handle), qt_edge(edge)) })
}

pub(crate) fn set_margin(handle: BorrowedWindow<'_>, edge: Edge, margin: i32) -> Result<()> {
    ensure(handle)?;
    unsafe { ffi::winio_layer_shell_set_margin(widget(handle), qt_edge(edge), margin) };
    Ok(())
}

pub(crate) fn exclusive_zone(handle: BorrowedWindow<'_>) -> Result<i32> {
    ensure(handle)?;
    Ok(unsafe { ffi::winio_layer_shell_exclusive_zone(widget(handle)) })
}

pub(crate) fn set_exclusive_zone(handle: BorrowedWindow<'_>, zone: i32) -> Result<()> {
    ensure(handle)?;
    unsafe { ffi::winio_layer_shell_set_exclusive_zone(widget(handle), zone) };
    Ok(())
}

pub(crate) fn keyboard_interactivity(handle: BorrowedWindow<'_>) -> Result<KeyboardInteractivity> {
    ensure(handle)?;
    Ok(
        match unsafe { ffi::winio_layer_shell_keyboard_interactivity(widget(handle)) } {
            1 => KeyboardInteractivity::Exclusive,
            2 => KeyboardInteractivity::OnDemand,
            _ => KeyboardInteractivity::None,
        },
    )
}

pub(crate) fn set_keyboard_interactivity(
    handle: BorrowedWindow<'_>,
    mode: KeyboardInteractivity,
) -> Result<()> {
    ensure(handle)?;
    let mode = match mode {
        KeyboardInteractivity::None => 0,
        KeyboardInteractivity::Exclusive => 1,
        KeyboardInteractivity::OnDemand => 2,
    };
    unsafe { ffi::winio_layer_shell_set_keyboard_interactivity(widget(handle), mode) };
    Ok(())
}

pub(crate) fn namespace(handle: BorrowedWindow<'_>) -> Result<Option<String>> {
    ensure(handle)?;
    let namespace = unsafe { ffi::winio_layer_shell_namespace(widget(handle)) };
    Ok((!namespace.is_empty()).then_some(namespace))
}

pub(crate) fn set_namespace(handle: BorrowedWindow<'_>, namespace: &str) -> Result<()> {
    ensure(handle)?;
    unsafe { ffi::winio_layer_shell_set_namespace(widget(handle), namespace) };
    Ok(())
}

pub(crate) fn request_refresh(handle: BorrowedWindow<'_>) -> Result<()> {
    unsafe { ffi::winio_layer_shell_request_refresh(widget(handle)) };
    Ok(())
}

pub(crate) fn is_configured(handle: BorrowedWindow<'_>) -> bool {
    unsafe { ffi::winio_layer_shell_is_configured(widget(handle)) }
}

pub(crate) fn output_info(handle: BorrowedWindow<'_>) -> Option<crate::OutputInfo> {
    let name = unsafe { ffi::winio_layer_shell_output_name(widget(handle)) };
    (!name.is_empty()).then_some(crate::OutputInfo {
        name: Some(name),
        is_primary: true,
    })
}

fn qt_edge(edge: Edge) -> i32 {
    match edge {
        Edge::Top => 0,
        Edge::Bottom => 1,
        Edge::Left => 2,
        Edge::Right => 3,
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-layer-shell/src/qt.hpp");

        type QWidget;

        fn winio_layer_shell_is_supported() -> bool;

        unsafe fn winio_layer_shell_init(widget: *mut QWidget) -> i32;
        unsafe fn winio_layer_shell_layer(widget: *mut QWidget) -> i32;
        unsafe fn winio_layer_shell_set_layer(widget: *mut QWidget, layer: i32);
        unsafe fn winio_layer_shell_anchors(widget: *mut QWidget) -> u32;
        unsafe fn winio_layer_shell_set_anchors(widget: *mut QWidget, anchors: u32);
        unsafe fn winio_layer_shell_margin(widget: *mut QWidget, edge: i32) -> i32;
        unsafe fn winio_layer_shell_set_margin(widget: *mut QWidget, edge: i32, margin: i32);
        unsafe fn winio_layer_shell_exclusive_zone(widget: *mut QWidget) -> i32;
        unsafe fn winio_layer_shell_set_exclusive_zone(widget: *mut QWidget, zone: i32);
        unsafe fn winio_layer_shell_keyboard_interactivity(widget: *mut QWidget) -> i32;
        unsafe fn winio_layer_shell_set_keyboard_interactivity(widget: *mut QWidget, mode: i32);
        unsafe fn winio_layer_shell_namespace(widget: *mut QWidget) -> String;
        unsafe fn winio_layer_shell_set_namespace(widget: *mut QWidget, scope: &str);
        unsafe fn winio_layer_shell_is_configured(widget: *mut QWidget) -> bool;
        unsafe fn winio_layer_shell_request_refresh(widget: *mut QWidget);
        unsafe fn winio_layer_shell_output_name(widget: *mut QWidget) -> String;
    }
}
