use std::cell::{Cell, RefCell};

use compio_log::*;
use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_app_kit::{NSEvent, NSEventType, NSGraphicsContext, NSView};
use objc2_foundation::{MainThreadMarker, NSRect};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{KeyCode, MouseButton, Point, Size, Vector};

use crate::{
    ContextOwner, DrawAction, DrawingContext, GlobalRuntime, Result, Widget, catch,
    platform::Keyboard, transform_cgpoint,
};

#[derive(Debug)]
pub(crate) struct CanvasImpl {
    view: Retained<CanvasView>,
    handle: Widget,
}
#[inherit_methods(from = "self.handle")]
impl CanvasImpl {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let view = catch(|| CanvasView::new(parent.as_app_kit().mtm()))?;
        let handle = Widget::from_nsview(parent, view.clone().into_super())?;
        Ok(Self { view, handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.view.ivars().mouse_down.wait().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.view.ivars().mouse_up.wait().await
    }

    pub async fn wait_mouse_move(&self) -> Point {
        self.view.ivars().mouse_move.wait().await;
        self.view
            .window()
            .map(|w| {
                let p = w.mouseLocationOutsideOfEventStream();
                let p = self.view.convertPoint_fromView(p, None);
                transform_cgpoint(self.size().unwrap_or_default(), p)
            })
            .unwrap_or_default()
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        self.view.ivars().mouse_scroll.wait().await
    }

    pub async fn wait_key_down(&self) -> KeyCode {
        self.view.ivars().keyboard.wait_key_down().await
    }

    pub async fn wait_key_up(&self) -> KeyCode {
        self.view.ivars().keyboard.wait_key_up().await
    }

    pub async fn wait_key_char(&self) -> char {
        self.view.ivars().keyboard.wait_key_char().await
    }
}

winio_handle::impl_as_widget!(CanvasImpl, handle);

#[derive(Debug)]
pub struct Canvas {
    handle: CanvasImpl,
}

#[inherit_methods(from = "self.handle")]
impl Canvas {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = CanvasImpl::new(parent)?;
        Ok(Self { handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn context(&mut self) -> Result<DrawingContext<'_>> {
        let size = self.size()?;
        let actions = self.handle.view.ivars().take_buffer();
        Ok(DrawingContext::new(size, self, actions))
    }

    pub async fn wait_mouse_down(&self) -> MouseButton {
        self.handle.wait_mouse_down().await
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.handle.wait_mouse_up().await
    }

    pub async fn wait_mouse_move(&self) -> Point {
        self.handle.wait_mouse_move().await
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        self.handle.wait_mouse_wheel().await
    }

    pub async fn wait_key_down(&self) -> KeyCode {
        self.handle.wait_key_down().await
    }

    pub async fn wait_key_up(&self) -> KeyCode {
        self.handle.wait_key_up().await
    }

    pub async fn wait_key_char(&self) -> char {
        self.handle.wait_key_char().await
    }
}

winio_handle::impl_as_widget!(Canvas, handle);

fn draw_rect(actions: &[DrawAction], _rect: NSRect, factor: f64) {
    let Some(ns_context) = NSGraphicsContext::currentContext() else {
        error!("Cannot get current NSGraphicsContext");
        return;
    };
    let context = ns_context.CGContext();
    DrawAction::draw_rect(actions, &context, factor);
}

#[derive(Debug, Default)]
struct CanvasViewIvars {
    mouse_down: Callback<MouseButton>,
    mouse_up: Callback<MouseButton>,
    mouse_move: Callback,
    mouse_scroll: Callback<Vector>,
    keyboard: Keyboard,
    actions: RefCell<Vec<DrawAction>>,
    // A buffer for actions, to avoid frequent allocations.
    actions_buf: RefCell<Vec<DrawAction>>,
    factor: Cell<f64>,
}

impl CanvasViewIvars {
    pub fn take_buffer(&self) -> Vec<DrawAction> {
        std::mem::take(&mut self.actions_buf.borrow_mut())
    }

    pub fn swap_buffer(&self, buf: &mut Vec<DrawAction>) {
        {
            let mut actions = self.actions.borrow_mut();
            std::mem::swap::<Vec<DrawAction>>(&mut actions, buf);
        }
        {
            let mut actions_buf = self.actions_buf.borrow_mut();
            std::mem::swap::<Vec<DrawAction>>(&mut actions_buf, buf);
            actions_buf.clear();
        }
    }
}

define_class! {
    #[unsafe(super(NSView))]
    #[name = "WinioCanvasView"]
    #[ivars = CanvasViewIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct CanvasView;

    #[allow(non_snake_case)]
    impl CanvasView {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(CanvasViewIvars::default());
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(acceptsFirstResponder))]
        unsafe fn acceptsFirstResponder(&self) -> bool {
            true
        }

        #[unsafe(method(becomeFirstResponder))]
        unsafe fn becomeFirstResponder(&self) -> bool {
            let accepted = unsafe { msg_send![super(self), becomeFirstResponder] };
            if accepted {
                self.ivars().keyboard.sync_modifiers(NSEvent::modifierFlags_class());
            }
            accepted
        }

        #[unsafe(method(keyDown:))]
        unsafe fn keyDown(&self, event: &NSEvent) {
            self.ivars().keyboard.key_down(event);
            self.ivars().keyboard.key_char(event);
        }

        #[unsafe(method(keyUp:))]
        unsafe fn keyUp(&self, event: &NSEvent) {
            self.ivars().keyboard.key_up(event);
        }

        #[unsafe(method(flagsChanged:))]
        unsafe fn flagsChanged(&self, event: &NSEvent) {
            self.ivars().keyboard.flags_changed(event.modifierFlags());
        }

        #[unsafe(method(drawRect:))]
        unsafe fn drawRect(&self, rect: NSRect) {
            let ivars = self.ivars();
            draw_rect(&ivars.actions.borrow(), rect, ivars.factor.get())
        }

        #[unsafe(method(mouseDown:))]
        unsafe fn mouseDown(&self, event: &NSEvent) {
            self.ivars().mouse_down.signal::<GlobalRuntime>(mouse_button(event));
        }

        #[unsafe(method(mouseUp:))]
        unsafe fn mouseUp(&self, event: &NSEvent) {
            self.ivars().mouse_up.signal::<GlobalRuntime>(mouse_button(event));
        }

        #[unsafe(method(mouseDragged:))]
        unsafe fn mouseDragged(&self, _event: &NSEvent) {
            self.ivars().mouse_move.signal::<GlobalRuntime>(());
        }

        #[unsafe(method(mouseMoved:))]
        unsafe fn mouseMoved(&self, _event: &NSEvent) {
            self.ivars().mouse_move.signal::<GlobalRuntime>(());
        }

        #[unsafe(method(scrollWheel:))]
        unsafe fn scrollWheel(&self, event: &NSEvent) {
            if event.r#type() == NSEventType::ScrollWheel {
                self.ivars().mouse_scroll.signal::<GlobalRuntime>(
                    Vector::new(event.scrollingDeltaX(), event.scrollingDeltaY())
                );
            }
        }
    }
}

impl CanvasView {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}

fn mouse_button(event: &NSEvent) -> MouseButton {
    match event.r#type() {
        NSEventType::LeftMouseDown | NSEventType::LeftMouseUp => MouseButton::Left,
        NSEventType::RightMouseDown | NSEventType::RightMouseUp => MouseButton::Right,
        _ => MouseButton::Other,
    }
}

impl ContextOwner for Canvas {
    fn end_draw(&mut self, mut actions: Vec<DrawAction>) -> Result<()> {
        let ivars = self.handle.view.ivars();
        ivars.swap_buffer(&mut actions);
        ivars.factor.set(
            self.handle
                .view
                .window()
                .map(|w| w.backingScaleFactor())
                .unwrap_or(1.0),
        );
        catch(|| self.handle.view.setNeedsDisplay(true))?;
        Ok(())
    }
}
