use std::{ptr::NonNull, rc::Rc};

use block2::StackBlock;
use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_foundation::{MainThreadMarker, NSArray, NSSize, NSString};
use objc2_ui_kit::{
    NSLayoutConstraint, UIColor, UITraitCollection, UIUserInterfaceStyle, UIView, UIViewController,
    UIWindow,
};
use winio_callback::Callback;
use winio_handle::{
    AsContainer, AsWidget, AsWindow, BorrowedContainer, BorrowedWidget, BorrowedWindow,
};
use winio_primitive::{Point, Size};
use winio_ui_apple_common::from_nsstring;

use crate::{
    Error, MOVE_SLAB, RESIZE_SLAB, Result, catch, first_ui_window_scene, from_cgpoint, from_cgsize,
    to_cgpoint, to_cgsize,
};

#[derive(Debug)]
pub struct Window {
    wnd: Retained<UIWindow>,
    content_view: Retained<UIView>,
    did_resize: Rc<Callback>,
    did_move: Rc<Callback>,
    resize_index: usize,
    move_index: usize,
}

impl Window {
    pub fn new() -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;

        catch(|| {
            let scene = first_ui_window_scene()?.ok_or(Error::NullPointer)?;

            let wnd = UIWindow::initWithWindowScene(UIWindow::alloc(mtm), &scene);

            if !cfg!(target_abi = "macabi") {
                fn provider(collection: NonNull<UITraitCollection>) -> NonNull<UIColor> {
                    fn provider_impl(collection: &UITraitCollection) -> Retained<UIColor> {
                        match unsafe { collection.userInterfaceStyle() } {
                            UIUserInterfaceStyle::Dark => UIColor::blackColor(),
                            _ => UIColor::whiteColor(),
                        }
                    }
                    unsafe {
                        NonNull::new_unchecked(Retained::into_raw(provider_impl(
                            collection.as_ref(),
                        )))
                    }
                }
                let provider = StackBlock::new(provider);
                let bg_color = unsafe { UIColor::colorWithDynamicProvider(&provider) };
                wnd.setBackgroundColor(Some(&bg_color));
            }

            let controller = UIViewController::new(mtm);

            let did_resize = Rc::new(Callback::new());
            let resize_index = RESIZE_SLAB.with_borrow_mut(|s| s.insert(did_resize.clone()));
            let did_move = Rc::new(Callback::new());
            let move_index = MOVE_SLAB.with_borrow_mut(|s| s.insert(did_move.clone()));

            wnd.setRootViewController(Some(&controller));
            wnd.makeKeyWindow();

            let root_view = controller.view().ok_or(Error::NullPointer)?;
            let content_view = UIView::new(mtm);
            root_view.addSubview(&content_view);
            content_view.setTranslatesAutoresizingMaskIntoConstraints(false);
            let c1 = content_view
                .leftAnchor()
                .constraintEqualToAnchor(&root_view.safeAreaLayoutGuide().leftAnchor());
            let c2 = content_view
                .rightAnchor()
                .constraintEqualToAnchor(&root_view.safeAreaLayoutGuide().rightAnchor());
            let c3 = content_view
                .topAnchor()
                .constraintEqualToAnchor(&root_view.safeAreaLayoutGuide().topAnchor());
            let c4 = content_view
                .bottomAnchor()
                .constraintEqualToAnchor(&root_view.safeAreaLayoutGuide().bottomAnchor());
            NSLayoutConstraint::activateConstraints(
                &NSArray::from_retained_slice(&[c1, c2, c3, c4]),
                mtm,
            );

            Ok(Self {
                wnd,
                content_view,
                did_resize,
                resize_index,
                did_move,
                move_index,
            })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool> {
        catch(|| !self.wnd.isHidden())
    }

    pub fn set_visible(&mut self, v: bool) -> Result<()> {
        catch(|| self.wnd.setHidden(!v))
    }

    pub fn loc(&self) -> Result<Point> {
        Ok(Point::zero())
    }

    pub fn set_loc(&mut self, _p: Point) -> Result<()> {
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        catch(|| {
            let frame = self.wnd.frame();
            from_cgsize(frame.size)
        })
    }

    pub fn set_size(&mut self, _v: Size) -> Result<()> {
        Ok(())
    }

    pub fn client_size(&self) -> Result<Size> {
        catch(|| from_cgsize(self.content_view.frame().size))
    }

    pub fn text(&self) -> Result<String> {
        catch(|| {
            self.wnd
                .windowScene()
                .map(|s| s.title())
                .as_deref()
                .map(from_nsstring)
                .unwrap_or_default()
        })
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        catch(|| {
            if let Some(scene) = self.wnd.windowScene() {
                scene.setTitle(Some(&NSString::from_str(s.as_ref())));
            }
        })
    }

    pub async fn wait_size(&self) {
        self.did_resize.wait().await;
    }

    pub async fn wait_move(&self) {
        self.did_move.wait().await;
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
        MOVE_SLAB.with_borrow_mut(|s| s.remove(self.move_index));
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
