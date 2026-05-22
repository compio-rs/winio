use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::AnyObject,
    sel,
};
use objc2_core_graphics::{CGAffineTransformIdentity, CGAffineTransformMakeRotation};
use objc2_foundation::NSObject;
use objc2_ui_kit::{UIControlEvents, UISlider};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Orient, Point, Size};

use crate::{GlobalRuntime, Result, catch, ui::Widget};

#[derive(Debug)]
struct ScrollBarImpl {
    handle: Widget,
    view: Retained<UISlider>,
    delegate: Retained<ScrollBarDelegate>,
    min: usize,
    max: usize,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBarImpl {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| unsafe {
            let view = UISlider::new(mtm);
            let handle = Widget::from_uiview(parent, Retained::cast_unchecked(view.clone()))?;

            let delegate = ScrollBarDelegate::new(mtm);
            let obj: &AnyObject = &delegate;
            view.addTarget_action_forControlEvents(
                Some(obj),
                sel!(onAction),
                UIControlEvents::ValueChanged,
            );

            view.setEnabled(true);

            Ok(Self {
                handle,
                view,
                delegate,
                min: 0,
                max: 0,
            })
        })
        .flatten()
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

    pub fn minimum(&self) -> Result<usize> {
        Ok(self.min)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let pos = self.pos()?;
        self.min = v;
        self.set_pos(pos)
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.max)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let pos = self.pos()?;
        self.max = v;
        self.set_pos(pos)
    }

    pub fn page(&self) -> Result<usize> {
        Ok(1)
    }

    pub fn set_page(&mut self, _v: usize) -> Result<()> {
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        catch(|| self.view.value() as _)
    }

    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        catch(|| self.view.setValue(v as f32))
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().on_move.wait().await
    }
}

winio_handle::impl_as_widget!(ScrollBarImpl, handle);

#[derive(Debug, Default)]
struct ScrollBarDelegateIvars {
    on_move: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioScrollBarDelegateUIKit"]
    #[ivars = ScrollBarDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct ScrollBarDelegate;

    #[allow(non_snake_case)]
    impl ScrollBarDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(ScrollBarDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(onAction))]
        unsafe fn onAction(&self) {
            self.ivars().on_move.signal::<GlobalRuntime>(());
        }
    }
}

impl ScrollBarDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}

#[derive(Debug)]
pub struct ScrollBar {
    handle: ScrollBarImpl,
    vertical: bool,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = ScrollBarImpl::new(&parent)?;
        Ok(Self {
            handle,
            vertical: false,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        if self.vertical {
            Ok(Size::new(20.0, 0.0))
        } else {
            Ok(Size::new(0.0, 20.0))
        }
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn orient(&self) -> Result<Orient> {
        if self.vertical {
            Ok(Orient::Vertical)
        } else {
            Ok(Orient::Horizontal)
        }
    }

    pub fn set_orient(&mut self, v: Orient) -> Result<()> {
        self.vertical = matches!(v, Orient::Vertical);
        if self.vertical {
            self.handle
                .view
                .setTransform(CGAffineTransformMakeRotation(std::f64::consts::FRAC_PI_2));
        } else {
            self.handle
                .view
                .setTransform(unsafe { CGAffineTransformIdentity });
        }
        Ok(())
    }

    pub fn minimum(&self) -> Result<usize>;

    pub fn set_minimum(&mut self, v: usize) -> Result<()>;

    pub fn maximum(&self) -> Result<usize>;

    pub fn set_maximum(&mut self, v: usize) -> Result<()>;

    pub fn page(&self) -> Result<usize>;

    pub fn set_page(&mut self, v: usize) -> Result<()>;

    pub fn pos(&self) -> Result<usize>;

    pub fn set_pos(&mut self, v: usize) -> Result<()>;

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }
}

winio_handle::impl_as_widget!(ScrollBar, handle);
