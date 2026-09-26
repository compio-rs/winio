use std::cell::{Cell, RefCell};

use objc2_core_graphics::CGContext;

use crate::DrawAction;

/// The drawing state of a canvas view shared by the AppKit and UIKit backends.
///
/// It keeps the actions of the last frame, a buffer reused to record the next
/// frame without frequent allocations, and the scale factor to draw with.
#[derive(Debug, Default)]
pub struct CanvasState {
    actions: RefCell<Vec<Box<dyn DrawAction>>>,
    // A buffer for actions, to avoid frequent allocations.
    actions_buf: RefCell<Vec<Box<dyn DrawAction>>>,
    factor: Cell<f64>,
}

impl CanvasState {
    /// Takes the buffer for recording the actions of a new frame.
    pub(crate) fn take_buffer(&self) -> Vec<Box<dyn DrawAction>> {
        std::mem::take(&mut self.actions_buf.borrow_mut())
    }

    /// Draws the actions of the last frame into `context`.
    pub fn draw_rect(&self, context: &CGContext) {
        crate::draw_rect(&self.actions.borrow(), context, self.factor.get());
    }

    /// Replaces the actions of the last frame with `actions` and stores the
    /// scale `factor` to draw them with.
    pub(crate) fn end_draw(&self, actions: Vec<Box<dyn DrawAction>>, factor: f64) {
        let old = self.actions.replace(actions);
        let mut buf = self.actions_buf.borrow_mut();
        *buf = old;
        buf.clear();
        self.factor.set(factor);
    }
}
