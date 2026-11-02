use winio::{handle::BorrowedWindow, primitive::Margin};

use crate::{Anchors, Error, KeyboardInteractivity, Layer, Result};

pub(crate) fn ensure(_window: BorrowedWindow<'_>) -> Result<()> {
    not_supported()
}

pub(crate) fn layer(_window: BorrowedWindow<'_>) -> Result<Layer> {
    not_supported()
}

pub(crate) fn set_layer(_window: BorrowedWindow<'_>, _layer: Layer) -> Result<()> {
    not_supported()
}

pub(crate) fn anchors(_window: BorrowedWindow<'_>) -> Result<Anchors> {
    not_supported()
}

pub(crate) fn set_anchors(_window: BorrowedWindow<'_>, _anchors: Anchors) -> Result<()> {
    not_supported()
}

pub(crate) fn margin(_window: BorrowedWindow<'_>) -> Result<Margin> {
    not_supported()
}

pub(crate) fn set_margin(_window: BorrowedWindow<'_>, _margin: Margin) -> Result<()> {
    not_supported()
}

pub(crate) fn exclusive_zone(_window: BorrowedWindow<'_>) -> Result<Option<f64>> {
    not_supported()
}

pub(crate) fn set_exclusive_zone(_window: BorrowedWindow<'_>, _zone: Option<f64>) -> Result<()> {
    not_supported()
}

pub(crate) fn keyboard_interactivity(_window: BorrowedWindow<'_>) -> Result<KeyboardInteractivity> {
    not_supported()
}

pub(crate) fn set_keyboard_interactivity(
    _window: BorrowedWindow<'_>,
    _mode: KeyboardInteractivity,
) -> Result<()> {
    not_supported()
}

pub(crate) fn namespace(_window: BorrowedWindow<'_>) -> Result<String> {
    not_supported()
}

pub(crate) fn set_namespace(_window: BorrowedWindow<'_>, _namespace: &str) -> Result<()> {
    not_supported()
}

fn not_supported<T>() -> Result<T> {
    Err(Error::NotSupported)
}
