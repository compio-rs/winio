use core::pin::Pin;

use bitflags::bitflags;
use cxx::{ExternType, type_id};
use winio_handle::BorrowedWindow;
use winio_primitive::Margin;

use crate::{Anchors, Error, KeyboardInteractivity, Layer, Result};

#[repr(C)]
pub(crate) struct QMargins {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

unsafe impl ExternType for QMargins {
    type Id = type_id!("QMargins");
    type Kind = cxx::kind::Trivial;
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum WinioLayerShellInit {
    Ok           = 0,
    NotSupported = 1,
    NoWindow     = 2,
    AlreadyShown = 3,
}

unsafe impl ExternType for WinioLayerShellInit {
    type Id = type_id!("WinioLayerShellInit");
    type Kind = cxx::kind::Trivial;
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WinioLayerShellLayer {
    Background = 0,
    Bottom     = 1,
    Top        = 2,
    Overlay    = 3,
}

unsafe impl ExternType for WinioLayerShellLayer {
    type Id = type_id!("WinioLayerShellLayer");
    type Kind = cxx::kind::Trivial;
}

impl From<Layer> for WinioLayerShellLayer {
    fn from(layer: Layer) -> Self {
        match layer {
            Layer::Background => Self::Background,
            Layer::Bottom => Self::Bottom,
            Layer::Top => Self::Top,
            Layer::Overlay => Self::Overlay,
        }
    }
}

impl From<WinioLayerShellLayer> for Layer {
    fn from(layer: WinioLayerShellLayer) -> Self {
        match layer {
            WinioLayerShellLayer::Background => Self::Background,
            WinioLayerShellLayer::Bottom => Self::Bottom,
            WinioLayerShellLayer::Top => Self::Top,
            WinioLayerShellLayer::Overlay => Self::Overlay,
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WinioLayerShellKeyboardInteractivity {
    None      = 0,
    Exclusive = 1,
    OnDemand  = 2,
}

unsafe impl ExternType for WinioLayerShellKeyboardInteractivity {
    type Id = type_id!("WinioLayerShellKeyboardInteractivity");
    type Kind = cxx::kind::Trivial;
}

impl From<KeyboardInteractivity> for WinioLayerShellKeyboardInteractivity {
    fn from(mode: KeyboardInteractivity) -> Self {
        match mode {
            KeyboardInteractivity::None => Self::None,
            KeyboardInteractivity::OnDemand => Self::OnDemand,
            KeyboardInteractivity::Exclusive => Self::Exclusive,
        }
    }
}

impl From<WinioLayerShellKeyboardInteractivity> for KeyboardInteractivity {
    fn from(mode: WinioLayerShellKeyboardInteractivity) -> Self {
        match mode {
            WinioLayerShellKeyboardInteractivity::None => Self::None,
            WinioLayerShellKeyboardInteractivity::OnDemand => Self::OnDemand,
            WinioLayerShellKeyboardInteractivity::Exclusive => Self::Exclusive,
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct WinioLayerShellAnchors: i32 {
        const Top = 1;
        const Bottom = 1 << 1;
        const Left = 1 << 2;
        const Right = 1 << 3;
    }
}

unsafe impl ExternType for WinioLayerShellAnchors {
    type Id = type_id!("WinioLayerShellAnchors");
    type Kind = cxx::kind::Trivial;
}

impl From<Anchors> for WinioLayerShellAnchors {
    fn from(anchors: Anchors) -> Self {
        Self::from_bits_retain(i32::from(anchors.bits()))
    }
}

impl From<WinioLayerShellAnchors> for Anchors {
    fn from(anchors: WinioLayerShellAnchors) -> Self {
        Self::from_bits_truncate(anchors.bits() as u8)
    }
}

pub(crate) fn ensure(handle: BorrowedWindow<'_>) -> Result<()> {
    window(handle).map(drop)
}

pub(crate) fn layer(handle: BorrowedWindow<'_>) -> Result<Layer> {
    if !is_layer_window(handle) {
        return Err(Error::NotSupported);
    }
    Ok(window(handle)?.layer().into())
}

pub(crate) fn set_layer(handle: BorrowedWindow<'_>, layer: Layer) -> Result<()> {
    window(handle)?.setLayer(layer.into());
    Ok(())
}

pub(crate) fn anchors(handle: BorrowedWindow<'_>) -> Result<Anchors> {
    if !is_layer_window(handle) {
        return Err(Error::NotSupported);
    }
    Ok(window(handle)?.anchors().into())
}

pub(crate) fn set_anchors(handle: BorrowedWindow<'_>, anchors: Anchors) -> Result<()> {
    window(handle)?.setAnchors(anchors.into());
    Ok(())
}

pub(crate) fn margin(handle: BorrowedWindow<'_>) -> Result<Margin> {
    if !is_layer_window(handle) {
        return Err(Error::NotSupported);
    }
    let margins = window(handle)?.margins();
    Ok(Margin::new(
        margins.top as f64,
        margins.right as f64,
        margins.bottom as f64,
        margins.left as f64,
    ))
}

pub(crate) fn set_margin(handle: BorrowedWindow<'_>, margin: Margin) -> Result<()> {
    let window = window(handle)?;
    window.setMargins(&QMargins {
        left: margin.left as i32,
        top: margin.top as i32,
        right: margin.right as i32,
        bottom: margin.bottom as i32,
    });
    Ok(())
}

pub(crate) fn exclusive_zone(handle: BorrowedWindow<'_>) -> Result<Option<f64>> {
    if !is_layer_window(handle) {
        return Err(Error::NotSupported);
    }
    let zone = window(handle)?.exclusionZone();
    Ok((zone >= 0).then_some(zone as f64))
}

pub(crate) fn set_exclusive_zone(handle: BorrowedWindow<'_>, zone: Option<f64>) -> Result<()> {
    window(handle)?.setExclusiveZone(zone.map_or(-1, |zone| zone as i32));
    Ok(())
}

pub(crate) fn keyboard_interactivity(handle: BorrowedWindow<'_>) -> Result<KeyboardInteractivity> {
    if !is_layer_window(handle) {
        return Err(Error::NotSupported);
    }
    Ok(window(handle)?.keyboardInteractivity().into())
}

pub(crate) fn set_keyboard_interactivity(
    handle: BorrowedWindow<'_>,
    mode: KeyboardInteractivity,
) -> Result<()> {
    window(handle)?.setKeyboardInteractivity(mode.into());
    Ok(())
}

pub(crate) fn namespace(handle: BorrowedWindow<'_>) -> Result<String> {
    if !is_layer_window(handle) {
        return Err(Error::NotSupported);
    }
    let window = window(handle)?;
    Ok(ffi::winio_layer_shell_scope(&window))
}

pub(crate) fn set_namespace(handle: BorrowedWindow<'_>, namespace: &str) -> Result<()> {
    // The scope is passed when the layer surface is created, so it cannot be
    // changed after the window has been shown.
    let widget = widget(handle)?;
    if ffi::winio_layer_shell_is_shown(&widget) {
        return Err(Error::AlreadyShown);
    }
    let window = window(handle)?;
    ffi::winio_layer_shell_set_scope(window, namespace);
    Ok(())
}

fn widget(handle: BorrowedWindow<'_>) -> Result<Pin<&mut ffi::QWidget>> {
    let widget = unsafe { handle.as_qt::<ffi::QWidget>().as_mut() }.ok_or(Error::NoWindow)?;
    Ok(unsafe { Pin::new_unchecked(widget) })
}

/// Whether the window has already been initialized as a layer-shell surface.
///
/// Used by the getters so they report [`Error::NotSupported`] instead of
/// silently turning the window into a layer surface.
fn is_layer_window(handle: BorrowedWindow<'_>) -> bool {
    match widget(handle) {
        Ok(widget) => ffi::winio_layer_shell_is_initialized(&widget),
        Err(_) => false,
    }
}

fn window(handle: BorrowedWindow<'_>) -> Result<Pin<&mut ffi::WinioLayerShellWindow>> {
    let mut widget = widget(handle)?;
    match ffi::winio_layer_shell_init(widget.as_mut()) {
        WinioLayerShellInit::Ok => {}
        WinioLayerShellInit::NotSupported => return Err(Error::NotSupported),
        WinioLayerShellInit::NoWindow => return Err(Error::NoWindow),
        WinioLayerShellInit::AlreadyShown => return Err(Error::AlreadyShown),
    }
    let window = unsafe { ffi::winio_layer_shell(widget).as_mut() }.ok_or(Error::NoWindow)?;
    Ok(unsafe { Pin::new_unchecked(window) })
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-layer-shell/src/qt.hpp");

        type QWidget;
        type WinioLayerShellWindow;
        type QMargins = super::QMargins;
        type WinioLayerShellInit = super::WinioLayerShellInit;
        type WinioLayerShellLayer = super::WinioLayerShellLayer;
        type WinioLayerShellKeyboardInteractivity = super::WinioLayerShellKeyboardInteractivity;
        type WinioLayerShellAnchors = super::WinioLayerShellAnchors;

        fn winio_layer_shell_is_initialized(widget: &QWidget) -> bool;
        fn winio_layer_shell_is_shown(widget: &QWidget) -> bool;
        fn winio_layer_shell_init(widget: Pin<&mut QWidget>) -> WinioLayerShellInit;

        unsafe fn winio_layer_shell(widget: Pin<&mut QWidget>) -> *mut WinioLayerShellWindow;

        fn layer(self: &WinioLayerShellWindow) -> WinioLayerShellLayer;
        fn setLayer(self: Pin<&mut WinioLayerShellWindow>, layer: WinioLayerShellLayer);

        fn anchors(self: &WinioLayerShellWindow) -> WinioLayerShellAnchors;
        fn setAnchors(self: Pin<&mut WinioLayerShellWindow>, anchors: WinioLayerShellAnchors);

        fn margins(self: &WinioLayerShellWindow) -> QMargins;
        fn setMargins(self: Pin<&mut WinioLayerShellWindow>, margins: &QMargins);

        fn exclusionZone(self: &WinioLayerShellWindow) -> i32;
        fn setExclusiveZone(self: Pin<&mut WinioLayerShellWindow>, zone: i32);

        fn keyboardInteractivity(
            self: &WinioLayerShellWindow,
        ) -> WinioLayerShellKeyboardInteractivity;
        fn setKeyboardInteractivity(
            self: Pin<&mut WinioLayerShellWindow>,
            interactivity: WinioLayerShellKeyboardInteractivity,
        );

        fn winio_layer_shell_scope(window: &WinioLayerShellWindow) -> String;
        fn winio_layer_shell_set_scope(window: Pin<&mut WinioLayerShellWindow>, scope: &str);
    }
}
