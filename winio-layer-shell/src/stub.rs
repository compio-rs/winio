use winio::handle::BorrowedWindow;

use crate::{Anchors, Edge, Error, KeyboardInteractivity, Layer, Result};

pub(crate) fn is_supported() -> bool {
    false
}

pub(crate) fn ensure(_window: BorrowedWindow<'_>) -> Result<()> {
    Err(Error::NotSupported)
}

pub(crate) fn layer(_window: BorrowedWindow<'_>) -> Result<Layer> {
    Err(Error::NotSupported)
}

pub(crate) fn set_layer(_window: BorrowedWindow<'_>, _layer: Layer) -> Result<()> {
    Err(Error::NotSupported)
}

pub(crate) fn anchors(_window: BorrowedWindow<'_>) -> Result<Anchors> {
    Err(Error::NotSupported)
}

pub(crate) fn set_anchors(_window: BorrowedWindow<'_>, _anchors: Anchors) -> Result<()> {
    Err(Error::NotSupported)
}

pub(crate) fn margin(_window: BorrowedWindow<'_>, _edge: Edge) -> Result<i32> {
    Err(Error::NotSupported)
}

pub(crate) fn set_margin(_window: BorrowedWindow<'_>, _edge: Edge, _margin: i32) -> Result<()> {
    Err(Error::NotSupported)
}

pub(crate) fn exclusive_zone(_window: BorrowedWindow<'_>) -> Result<i32> {
    Err(Error::NotSupported)
}

pub(crate) fn set_exclusive_zone(_window: BorrowedWindow<'_>, _zone: i32) -> Result<()> {
    Err(Error::NotSupported)
}

pub(crate) fn keyboard_interactivity(_window: BorrowedWindow<'_>) -> Result<KeyboardInteractivity> {
    Err(Error::NotSupported)
}

pub(crate) fn set_keyboard_interactivity(
    _window: BorrowedWindow<'_>,
    _mode: KeyboardInteractivity,
) -> Result<()> {
    Err(Error::NotSupported)
}

pub(crate) fn request_refresh(_window: BorrowedWindow<'_>) -> Result<()> {
    Err(Error::NotSupported)
}

pub(crate) fn is_configured(_window: BorrowedWindow<'_>) -> bool {
    false
}

pub(crate) fn output_info(_window: BorrowedWindow<'_>) -> Option<crate::OutputInfo> {
    None
}

pub(crate) fn namespace(_window: BorrowedWindow<'_>) -> Result<Option<String>> {
    Err(Error::NotSupported)
}

pub(crate) fn set_namespace(_window: BorrowedWindow<'_>, _namespace: &str) -> Result<()> {
    Err(Error::NotSupported)
}
