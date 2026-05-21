use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained, Weak},
};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSSize};
use objc2_ui_kit::{UIScreen, UIView, UIViewController, UIWindow};
use winio_callback::Callback;
use winio_handle::{
    AsContainer, AsWidget, AsWindow, BorrowedContainer, BorrowedWidget, BorrowedWindow,
};
use winio_primitive::{Point, Rect, Size};

use crate::{
    Error, Result, catch,
    ui::{from_cgsize, to_cgsize, transform_cgrect, transform_rect},
};

#[derive(Debug)]
pub struct Window {
    wnd: Retained<UIWindow>,
    content_view: Retained<UIView>,
    delegate: Retained<WindowDelegate>,
}

impl Window {
    pub fn new() -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;

        let mut this = catch(|| {
            let scene = crate::current_scene()?;

            let scene = Retained::downcast(scene).map_err(|_| Error::NullPointer)?;

            let wnd = UIWindow::initWithWindowScene(UIWindow::alloc(mtm), &scene);

            let controller = UIViewController::new(mtm);
            wnd.setRootViewController(Some(&controller));
            wnd.makeKeyWindow();

            let delegate = WindowDelegate::new(mtm);

            let content_view = controller.view().ok_or(Error::NullPointer)?;

            Ok(Self {
                wnd,
                content_view,
                delegate,
            })
        })
        .flatten()?;
        this.set_loc(Point::zero())?;
        Ok(this)
    }

    fn screen(&self) -> Retained<UIScreen> {
        self.wnd.screen()
    }

    pub fn is_visible(&self) -> Result<bool> {
        catch(|| !self.wnd.isHidden())
    }

    pub fn set_visible(&mut self, v: bool) -> Result<()> {
        catch(|| self.wnd.setHidden(!v))
    }

    pub fn loc(&self) -> Result<Point> {
        catch(|| {
            let frame = self.wnd.frame();
            let screen_frame = self.screen().bounds();
            Ok(transform_cgrect(from_cgsize(screen_frame.size), frame).origin)
        })
        .flatten()
    }

    pub fn set_loc(&mut self, p: Point) -> Result<()> {
        catch(|| {
            let frame = self.wnd.frame();
            let screen_frame = self.screen().bounds();
            let frame = transform_rect(
                from_cgsize(screen_frame.size),
                Rect::new(p, from_cgsize(frame.size)),
            );
            self.wnd.setFrame(frame);
        })
    }

    pub fn size(&self) -> Result<Size> {
        catch(|| {
            let frame = self.wnd.frame();
            from_cgsize(frame.size)
        })
    }

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        catch(|| {
            let mut frame = self.wnd.frame();
            frame.size = to_cgsize(v);
            self.wnd.setFrame(frame);
        })
    }

    pub fn client_size(&self) -> Result<Size> {
        catch(|| {
            let frame = self.wnd.frame();
            from_cgsize(frame.size)
        })
    }

    pub fn text(&self) -> Result<String> {
        Ok(String::new())
    }

    pub fn set_text(&mut self, _s: impl AsRef<str>) -> Result<()> {
        Ok(())
    }

    pub async fn wait_size(&self) {
        self.delegate.ivars().did_resize.wait().await
    }

    pub async fn wait_move(&self) {
        self.delegate.ivars().did_move.wait().await
    }

    pub async fn wait_close(&self) {
        self.delegate.ivars().should_close.wait().await
    }

    pub async fn wait_theme_changed(&self) {
        self.delegate.ivars().defaults_change.wait().await
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        BorrowedWindow::ui_kit(&self.wnd)
    }
}

impl AsContainer for Window {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::ui_kit(&self.content_view)
    }
}

#[derive(Debug, Default)]
struct WindowDelegateIvars {
    did_resize: Callback,
    did_move: Callback,
    should_close: Callback,
    defaults_change: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioWindowDelegateUIKit"]
    #[ivars = WindowDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct WindowDelegate;

    #[allow(non_snake_case)]
    impl WindowDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(WindowDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for WindowDelegate {}
}

impl WindowDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Vibrancy {}

#[derive(Debug)]
pub(crate) struct Widget {
    parent: Weak<UIView>,
    view: Retained<UIView>,
}

impl Widget {
    pub fn from_uiview(parent: impl AsContainer, view: Retained<UIView>) -> Result<Self> {
        let mut this = catch(|| {
            let parent = parent.as_container().as_ui_kit();
            parent.addSubview(&view);
            Self {
                parent: Weak::from_retained(parent),
                view,
            }
        })?;
        this.set_loc(Point::zero())?;
        Ok(this)
    }

    pub fn parent(&self) -> Result<Retained<UIView>> {
        catch(|| self.parent.load())?.ok_or(Error::NullPointer)
    }

    pub fn is_visible(&self) -> Result<bool> {
        catch(|| !self.view.isHidden())
    }

    pub fn set_visible(&mut self, v: bool) -> Result<()> {
        catch(|| self.view.setHidden(!v))
    }

    pub fn is_enabled(&self) -> Result<bool> {
        catch(|| self.view.isUserInteractionEnabled())
    }

    pub fn set_enabled(&mut self, v: bool) -> Result<()> {
        catch(|| self.view.setUserInteractionEnabled(v))
    }

    pub fn preferred_size(&self) -> Result<Size> {
        catch(|| {
            let s = self.view.sizeThatFits(NSSize::ZERO);
            from_cgsize(s)
        })
    }

    pub fn loc(&self) -> Result<Point> {
        catch(|| {
            let frame = self.view.frame();
            let parent_frame = self.parent()?.frame();
            Ok(transform_cgrect(from_cgsize(parent_frame.size), frame).origin)
        })
        .flatten()
    }

    pub fn set_loc(&mut self, p: Point) -> Result<()> {
        catch(|| {
            let frame = self.view.frame();
            let parent_frame = self.parent()?.frame();
            let frame = transform_rect(
                from_cgsize(parent_frame.size),
                Rect::new(p, from_cgsize(frame.size)),
            );
            self.view.setFrame(frame);
            Ok(())
        })
        .flatten()
    }

    pub fn size(&self) -> Result<Size> {
        catch(|| {
            let frame = self.view.frame();
            from_cgsize(frame.size)
        })
    }

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        catch(|| {
            let mut frame = self.view.frame();
            frame.size = to_cgsize(v);
            self.view.setFrame(frame);
        })
    }

    pub fn tooltip(&self) -> Result<String> {
        Ok(String::new())
    }

    pub fn set_tooltip(&mut self, _s: impl AsRef<str>) -> Result<()> {
        Ok(())
    }
}

impl Drop for Widget {
    fn drop(&mut self) {
        self.view.removeFromSuperview();
    }
}

impl AsWidget for Widget {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        BorrowedWidget::ui_kit(&self.view)
    }
}

impl AsContainer for Widget {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::ui_kit(&self.view)
    }
}

#[derive(Debug)]
pub struct View {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl View {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        catch(|| {
            let parent = parent.as_container();
            let mtm = parent.as_ui_kit().mtm();

            let view = UIView::new(mtm);
            let handle =
                Widget::from_uiview(parent, unsafe { Retained::cast_unchecked(view.clone()) })?;

            Ok(Self { handle })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;
}

winio_handle::impl_as_widget!(View, handle);
winio_handle::impl_as_container!(View, handle);
