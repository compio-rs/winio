use std::rc::Rc;

use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_core_foundation::CGPoint;
use objc2_foundation::{MainThreadMarker, NSSize};
use objc2_ui_kit::{UIView, UIViewController, UIWindow};
use winio_callback::Callback;
use winio_handle::{
    AsContainer, AsWidget, AsWindow, BorrowedContainer, BorrowedWidget, BorrowedWindow,
};
use winio_primitive::{Point, Size};

use crate::{
    Error, RESIZE_SLAB, Result, catch, first_ui_window_scene, from_cgpoint, to_cgpoint,
    ui::{from_cgrect, from_cgsize, to_cgsize},
};

#[derive(Debug)]
pub struct Window {
    wnd: Retained<UIWindow>,
    content_view: Retained<UIView>,
    did_resize: Rc<Callback<Size>>,
    resize_index: usize,
}

impl Window {
    pub fn new() -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;

        catch(|| {
            let scene = first_ui_window_scene()?.ok_or(Error::NullPointer)?;

            let wnd = UIWindow::initWithWindowScene(UIWindow::alloc(mtm), &scene);

            let controller = UIViewController::new(mtm);

            let did_resize = Rc::new(Callback::new());
            let index = RESIZE_SLAB.with_borrow_mut(|s| s.insert(did_resize.clone()));

            wnd.setRootViewController(Some(&controller));
            wnd.makeKeyWindow();

            let content_view = controller.view().ok_or(Error::NullPointer)?;

            Ok(Self {
                wnd,
                content_view,
                did_resize,
                resize_index: index,
            })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool> {
        catch(|| !self.wnd.isHidden())
    }

    pub fn set_visible(&mut self, v: bool) -> Result<()> {
        catch(|| {
            self.wnd.setHidden(!v);
            #[cfg(target_abi = "macabi")]
            {
                if v {
                    use objc2_ui_kit::{UIApplication, UISceneSessionActivationRequest};

                    let mtm = self.wnd.mtm();
                    let app = UIApplication::sharedApplication(mtm);
                    let request = unsafe { UISceneSessionActivationRequest::new() };
                    let activity = self
                        .wnd
                        .windowScene()
                        .and_then(|scene| scene.userActivity());
                    request.setUserActivity(activity.as_deref());
                    app.activateSceneSessionForRequest_errorHandler(&request, None);
                }
            }
        })
    }

    pub fn loc(&self) -> Result<Point> {
        catch(|| {
            let frame = self.wnd.frame();
            Ok(from_cgrect(frame).origin)
        })
        .flatten()
    }

    pub fn set_loc(&mut self, p: Point) -> Result<()> {
        catch(|| {
            let mut frame = self.wnd.frame();
            frame.origin = to_cgpoint(p);
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
        catch(|| from_cgsize(self.content_view.frame().size))
    }

    pub fn text(&self) -> Result<String> {
        Ok(String::new())
    }

    pub fn set_text(&mut self, _s: impl AsRef<str>) -> Result<()> {
        Ok(())
    }

    pub async fn wait_size(&self) {
        let new_size = self.did_resize.wait().await;

        const TITLE_BAR_HEIGHT: f64 = 30.0;

        let mut frame = self.wnd.frame();
        frame.size = to_cgsize(new_size);
        self.wnd.setFrame(frame);
        frame.origin = CGPoint::new(0.0, TITLE_BAR_HEIGHT);
        frame.size.height -= TITLE_BAR_HEIGHT;
        self.content_view.setFrame(frame);
    }

    pub async fn wait_move(&self) {
        std::future::pending().await
    }

    pub async fn wait_close(&self) {
        std::future::pending().await
    }

    pub async fn wait_theme_changed(&self) {
        std::future::pending().await
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        RESIZE_SLAB.with_borrow_mut(|s| s.remove(self.resize_index));
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

#[derive(Debug)]
pub(crate) struct Widget {
    view: Retained<UIView>,
}

impl Widget {
    pub fn from_uiview(parent: impl AsContainer, view: Retained<UIView>) -> Result<Self> {
        catch(|| {
            let parent = parent.as_container().as_ui_kit();
            parent.addSubview(&view);
            Self { view }
        })
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
            from_cgpoint(frame.origin)
        })
    }

    pub fn set_loc(&mut self, p: Point) -> Result<()> {
        catch(|| {
            let mut frame = self.view.frame();
            frame.origin = to_cgpoint(p);
            self.view.setFrame(frame);
        })
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
