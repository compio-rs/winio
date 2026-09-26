use std::{
    cell::{Cell, RefCell},
    ptr::null_mut,
};

use compio_log::*;
use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_core_foundation::{CFRange, CFRetained, CGAffineTransform, CGPoint};
use objc2_core_graphics::{
    CGAffineTransformMake, CGAffineTransformMakeScale, CGColor, CGContext, CGMutablePath, CGPath,
};
use objc2_core_text::CTFramesetter;
use objc2_foundation::{MainThreadMarker, NSRect, NSSet, NSSize};
use objc2_ui_kit::{
    UIEvent, UIGraphicsGetCurrentContext, UIPress, UIPressesEvent, UITouch, UIView,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{
    BitmapRect, ColorTheme, Font, KeyCode, MouseButton, Point, Rect, RelativePoint, Size, Vector,
};

use crate::{
    ContextOwner, DrawAction, DrawingContext, GlobalRuntime, Result, Widget, catch,
    create_attr_str, from_cgsize, platform::Keyboard, to_cgpoint, to_cgrect, transform_cgpoint,
};

#[derive(Debug)]
pub(crate) struct CanvasImpl {
    view: Retained<CanvasView>,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl CanvasImpl {
    pub fn new(parent: impl AsContainer, flipped: bool) -> Result<Self> {
        let parent = parent.as_container();
        let view = catch(|| {
            let view = CanvasView::new(parent.as_ui_kit().mtm());
            if flipped {
                view.setTransform(CGAffineTransformMakeScale(1.0, -1.0));
            }
            view
        })?;
        let handle = Widget::from_uiview(parent, view.clone().into_super())?;
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
        self.view.ivars().touches_began.wait().await;
        MouseButton::Left
    }

    pub async fn wait_mouse_up(&self) -> MouseButton {
        self.view.ivars().touches_ended.wait().await;
        MouseButton::Left
    }

    pub async fn wait_mouse_move(&self) -> Point {
        let p = self.view.ivars().touches_moved.wait().await;
        let size = self.view.frame().size;
        let size = from_cgsize(size);
        transform_cgpoint(size, p)
    }

    pub async fn wait_mouse_wheel(&self) -> Vector {
        std::future::pending().await
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
        let handle = CanvasImpl::new(parent, true)?;
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

fn draw_rect(actions: &[Box<dyn DrawAction>], rect: NSRect, factor: f64) {
    let Some(context) = UIGraphicsGetCurrentContext() else {
        error!("Cannot get current CGContext");
        return;
    };
    if !matches!(crate::color_theme(), Ok(ColorTheme::Dark)) {
        CGContext::set_rgb_fill_color(Some(&context), 1.0, 1.0, 1.0, 1.0);
        CGContext::fill_rect(Some(&context), rect);
    } else {
        CGContext::clear_rect(Some(&context), rect);
    }
    winio_ui_apple_common::draw_rect(actions, &context, factor);
}

#[derive(Debug, Default)]
struct CanvasViewIvars {
    touches_began: Callback,
    touches_moved: Callback<CGPoint>,
    touches_ended: Callback,
    keyboard: Keyboard,
    actions: RefCell<Vec<Box<dyn DrawAction>>>,
    actions_buf: RefCell<Vec<Box<dyn DrawAction>>>,
    factor: Cell<f64>,
}

impl CanvasViewIvars {
    pub fn take_buffer(&self) -> Vec<Box<dyn DrawAction>> {
        std::mem::take(&mut self.actions_buf.borrow_mut())
    }

    pub fn swap_buffer(&self, buf: &mut Vec<Box<dyn DrawAction>>) {
        {
            let mut actions = self.actions.borrow_mut();
            std::mem::swap::<Vec<Box<dyn DrawAction>>>(&mut actions, buf);
        }
        {
            let mut actions_buf = self.actions_buf.borrow_mut();
            std::mem::swap::<Vec<Box<dyn DrawAction>>>(&mut actions_buf, buf);
            actions_buf.clear();
        }
    }
}

define_class! {
    #[unsafe(super(UIView))]
    #[name = "WinioCanvasViewUIKit"]
    #[ivars = CanvasViewIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct CanvasView;

    #[allow(non_snake_case)]
    impl CanvasView {
        #[unsafe(method_id(initWithFrame:))]
        fn initWithFrame(this: Allocated<Self>, frame: NSRect) -> Option<Retained<Self>> {
            let this = this.set_ivars(CanvasViewIvars::default());
            unsafe { msg_send![super(this), initWithFrame: frame] }
        }

        #[unsafe(method(canBecomeFirstResponder))]
        fn canBecomeFirstResponder(&self) -> bool {
            true
        }

        #[unsafe(method(pressesBegan:withEvent:))]
        unsafe fn pressesBegan(&self, presses: &NSSet<UIPress>, event: Option<&UIPressesEvent>) {
            let mut handled = false;
            for press in presses {
                if let Some(key) = press.key(self.mtm()) {
                    handled = true;
                    self.ivars().keyboard.key_down(&key);
                    self.ivars().keyboard.key_char(&key);
                }
            }
            if !handled {
                unsafe { msg_send![super(self), pressesBegan: presses, withEvent: event] }
            }
        }

        #[unsafe(method(pressesEnded:withEvent:))]
        unsafe fn pressesEnded(&self, presses: &NSSet<UIPress>, event: Option<&UIPressesEvent>) {
            let mut handled = false;
            for press in presses {
                if let Some(key) = press.key(self.mtm()) {
                    handled = true;
                    self.ivars().keyboard.key_up(&key);
                }
            }
            if !handled {
                unsafe { msg_send![super(self), pressesEnded: presses, withEvent: event] }
            }
        }

        #[unsafe(method(pressesCancelled:withEvent:))]
        unsafe fn pressesCancelled(&self, presses: &NSSet<UIPress>, event: Option<&UIPressesEvent>) {
            let mut handled = false;
            for press in presses {
                if let Some(key) = press.key(self.mtm()) {
                    handled = true;
                    self.ivars().keyboard.key_up(&key);
                }
            }
            if !handled {
                unsafe { msg_send![super(self), pressesCancelled: presses, withEvent: event] }
            }
        }

        #[unsafe(method(drawRect:))]
        unsafe fn drawRect(&self, rect: NSRect) {
            let ivars = self.ivars();
            draw_rect(&ivars.actions.borrow(), rect, ivars.factor.get())
        }

        #[unsafe(method(touchesBegan:withEvent:))]
        unsafe fn touchesBegan(&self, _touches: &NSSet<UITouch>, _event: &UIEvent) {
            self.becomeFirstResponder();
            self.ivars().touches_began.signal::<GlobalRuntime>(());
        }

        #[unsafe(method(touchesMoved:withEvent:))]
        unsafe fn touchesMoved(&self, touches: &NSSet<UITouch>, _event: &UIEvent) {
            if let Some(touch) = touches.iter().next() {
                self.ivars().touches_moved.signal::<GlobalRuntime>(touch.locationInView(Some(self)));
            }
        }

        #[unsafe(method(touchesEnded:withEvent:))]
        unsafe fn touchesEnded(&self, _touches: &NSSet<UITouch>, _event: &UIEvent) {
            self.ivars().touches_ended.signal::<GlobalRuntime>(());
        }
    }
}

impl CanvasView {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), initWithFrame: NSRect::ZERO] }
    }
}

impl ContextOwner for Canvas {
    fn path_arc(
        &self,
        size: Size,
        rect: Rect,
        start: f64,
        end: f64,
        pie: bool,
    ) -> CFRetained<CGMutablePath> {
        path_arc(size, rect, start, end, pie)
    }

    fn path_ellipse(&self, size: Size, rect: Rect) -> CFRetained<CGPath> {
        path_ellipse(size, rect)
    }

    fn path_line(&self, size: Size, start: Point, end: Point) -> CFRetained<CGMutablePath> {
        path_line(size, start, end)
    }

    fn path_rect(&self, size: Size, rect: Rect) -> CFRetained<CGPath> {
        path_rect(size, rect)
    }

    fn path_round_rect(&self, size: Size, rect: Rect, round: Size) -> CFRetained<CGPath> {
        path_round_rect(size, rect, round)
    }

    fn text_frame(
        &self,
        size: Size,
        font: Font,
        color: &CGColor,
        anchor: RelativePoint,
        pos: Point,
        text: &str,
    ) -> Result<(CFRetained<CTFramesetter>, NSRect)> {
        let (framesetter, rect) = measure_str(font, color, anchor, pos, text, size)?;
        Ok((framesetter, to_cgrect(rect)))
    }

    fn image_clip(&self, _image_size: Size, clip: BitmapRect) -> NSRect {
        to_cgrect(Rect::new(
            Point::new(clip.origin.x as f64, clip.origin.y as f64),
            Size::new(clip.size.width as f64, clip.size.height as f64),
        ))
    }

    fn end_draw(&mut self, mut actions: Vec<Box<dyn DrawAction>>) -> Result<()> {
        let ivars = self.handle.view.ivars();
        ivars.swap_buffer(&mut actions);
        ivars.factor.set(
            self.handle
                .view
                .window()
                .map(|w| w.screen().scale())
                .unwrap_or(1.0),
        );
        catch(|| self.handle.view.setNeedsDisplay())?;
        Ok(())
    }
}

fn flip_transform(s: Size) -> CGAffineTransform {
    CGAffineTransformMake(1.0, 0.0, 0.0, -1.0, 0.0, s.height)
}

fn path_arc(s: Size, rect: Rect, start: f64, end: f64, pie: bool) -> CFRetained<CGMutablePath> {
    let radius = rect.size / 2.0;
    let centerp = Point::new(rect.origin.x + radius.width, rect.origin.y + radius.height);
    let startp = Point::new(
        centerp.x + radius.width * start.cos(),
        centerp.y + radius.height * start.sin(),
    );

    let rate = radius.height / radius.width;
    let transform = CGAffineTransformMake(1.0, 0.0, 0.0, rate, 0.0, 0.0);
    let trivial_transform = flip_transform(s);

    unsafe {
        let path = CGMutablePath::new();
        let centerp = to_cgpoint(centerp);
        let startp = to_cgpoint(startp);
        if pie {
            CGMutablePath::move_to_point(Some(&path), &trivial_transform, centerp.x, centerp.y);
            CGMutablePath::add_line_to_point(
                Some(&path),
                &trivial_transform,
                startp.x,
                startp.y / rate,
            );
        } else {
            CGMutablePath::move_to_point(Some(&path), &trivial_transform, startp.x, startp.y);
        }
        CGMutablePath::add_arc(
            Some(&path),
            &transform,
            centerp.x,
            centerp.y / rate,
            radius.width,
            -start,
            -end,
            true,
        );
        if pie {
            CGMutablePath::close_subpath(Some(&path));
        }
        path
    }
}

fn path_ellipse(s: Size, rect: Rect) -> CFRetained<CGPath> {
    let rect = to_cgrect(rect);
    let transform = flip_transform(s);
    unsafe { CGPath::with_ellipse_in_rect(rect, &transform) }
}

fn path_line(s: Size, start: Point, end: Point) -> CFRetained<CGMutablePath> {
    unsafe {
        let path = CGMutablePath::new();
        let transform = flip_transform(s);
        let p = to_cgpoint(start);
        CGMutablePath::move_to_point(Some(&path), &transform, p.x, p.y);
        let p = to_cgpoint(end);
        CGMutablePath::add_line_to_point(Some(&path), &transform, p.x, p.y);
        path
    }
}

fn path_rect(s: Size, rect: Rect) -> CFRetained<CGPath> {
    let rect = to_cgrect(rect);
    let transform = flip_transform(s);
    unsafe { CGPath::with_rect(rect, &transform) }
}

fn path_round_rect(s: Size, rect: Rect, round: Size) -> CFRetained<CGPath> {
    let rect = to_cgrect(rect);
    let transform = flip_transform(s);
    unsafe { CGPath::with_rounded_rect(rect, round.width, round.height, &transform) }
}

fn measure_str(
    font: Font,
    color: &CGColor,
    anchor: RelativePoint,
    pos: Point,
    text: &str,
    bound: Size,
) -> Result<(CFRetained<CTFramesetter>, Rect)> {
    let astr = create_attr_str(&font, color, text)?;
    let framesetter = unsafe { CTFramesetter::with_attributed_string(&astr) };
    let size = from_cgsize(unsafe {
        framesetter.suggest_frame_size_with_constraints(
            CFRange::new(0, 0),
            None,
            NSSize::new(bound.width, bound.height + font.size),
            null_mut(),
        )
    });
    let x = pos.x - size.width * anchor.x;
    let y = bound.height - pos.y - size.height * (1.0 - anchor.y);
    Ok((framesetter, Rect::new(Point::new(x, y), size)))
}
