use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained, Weak},
    runtime::ProtocolObject,
    sel,
};
use objc2_app_kit::{
    NSAppKitVersionNumber, NSAppKitVersionNumber10_10, NSAppKitVersionNumber10_11,
    NSAppKitVersionNumber10_14, NSAutoresizingMaskOptions, NSBackingStoreType, NSControl, NSScreen,
    NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState,
    NSVisualEffectView, NSWindow, NSWindowDelegate, NSWindowOrderingMode, NSWindowStyleMask,
};
use objc2_foundation::{
    MainThreadMarker, NSDistributedNotificationCenter, NSNotification, NSObject, NSObjectProtocol,
    NSPoint, NSRect, NSSize, NSString, ns_string,
};
use winio_callback::Callback;
use winio_handle::{
    AsContainer, AsWidget, AsWindow, BorrowedContainer, BorrowedWidget, BorrowedWindow,
};
use winio_primitive::{Point, Rect, Size};

use crate::{
    Error, GlobalRuntime, Result, catch,
    ui::{from_cgsize, from_nsstring, to_cgsize, transform_cgrect, transform_rect},
};

#[derive(Debug)]
pub struct Window {
    wnd: Retained<NSWindow>,
    content_view: Retained<NSView>,
    delegate: Retained<WindowDelegate>,
    vibrancy: Option<Vibrancy>,
    vibrancy_view: Option<Retained<NSVisualEffectView>>,
}

impl Window {
    pub fn new() -> Result<Self> {
        unsafe {
            let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;

            let frame = NSRect::new(NSPoint::ZERO, NSSize::new(100.0, 100.0));

            let mut this = catch(|| {
                let wnd = {
                    NSWindow::initWithContentRect_styleMask_backing_defer(
                        mtm.alloc(),
                        frame,
                        NSWindowStyleMask::Titled
                            | NSWindowStyleMask::Closable
                            | NSWindowStyleMask::Resizable
                            | NSWindowStyleMask::Miniaturizable,
                        NSBackingStoreType::Buffered,
                        false,
                    )
                };

                let delegate = WindowDelegate::new(mtm);
                let del_obj = ProtocolObject::from_ref(&*delegate);
                wnd.setDelegate(Some(del_obj));
                wnd.setAcceptsMouseMovedEvents(true);
                wnd.makeKeyWindow();

                NSDistributedNotificationCenter::defaultCenter().addObserver_selector_name_object(
                    &delegate,
                    sel!(userDefaultsDidChange),
                    Some(ns_string!("AppleInterfaceThemeChangedNotification")),
                    None,
                );

                let content_view = wnd.contentView().ok_or(Error::NullPointer)?.clone();

                Ok(Self {
                    wnd,
                    content_view,
                    delegate,
                    vibrancy: None,
                    vibrancy_view: None,
                })
            })
            .flatten()?;
            this.set_loc(Point::zero())?;
            Ok(this)
        }
    }

    fn screen(&self) -> Result<Retained<NSScreen>> {
        catch(|| self.wnd.screen())?.ok_or(Error::NullPointer)
    }

    pub fn is_visible(&self) -> Result<bool> {
        catch(|| self.wnd.isVisible())
    }

    pub fn set_visible(&mut self, v: bool) -> Result<()> {
        catch(|| self.wnd.setIsVisible(v))
    }

    pub fn loc(&self) -> Result<Point> {
        catch(|| {
            let frame = self.wnd.frame();
            let screen_frame = self.screen()?.frame();
            Ok(transform_cgrect(from_cgsize(screen_frame.size), frame).origin)
        })
        .flatten()
    }

    pub fn set_loc(&mut self, p: Point) -> Result<()> {
        catch(|| {
            let frame = self.wnd.frame();
            let screen_frame = self.screen()?.frame();
            let frame = transform_rect(
                from_cgsize(screen_frame.size),
                Rect::new(p, from_cgsize(frame.size)),
            );
            self.wnd.setFrame_display(frame, true);
            Ok(())
        })
        .flatten()
    }

    pub fn size(&self) -> Result<Size> {
        catch(|| from_cgsize(self.wnd.frame().size))
    }

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        catch(|| {
            let mut frame = self.wnd.frame();
            let ydiff = v.height - frame.size.height;
            frame.size = to_cgsize(v);
            frame.origin.y -= ydiff;
            self.wnd.setFrame_display(frame, true);
        })
    }

    pub fn client_size(&self) -> Result<Size> {
        catch(|| from_cgsize(self.content_view.frame().size))
    }

    pub fn text(&self) -> Result<String> {
        catch(|| from_nsstring(&self.wnd.title()))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        catch(|| self.wnd.setTitle(&NSString::from_str(s.as_ref())))
    }

    pub fn vibrancy(&self) -> Result<Option<Vibrancy>> {
        Ok(self.vibrancy)
    }

    pub fn set_vibrancy(&mut self, v: Option<Vibrancy>) -> Result<()> {
        unsafe {
            if self.vibrancy == v {
                return Ok(());
            }
            if NSAppKitVersionNumber < NSAppKitVersionNumber10_10 {
                return Err(Error::NotSupported);
            }
            self.vibrancy = v;

            catch(|| {
                if let Some(v) = v {
                    let view = &self.content_view;
                    let bounds = view.bounds();
                    let vev: Retained<NSVisualEffectView> = NSVisualEffectView::initWithFrame(
                        self.wnd.mtm().alloc::<NSVisualEffectView>(),
                        bounds,
                    );
                    #[allow(deprecated)]
                    let m = if (v as u32 > 9 && NSAppKitVersionNumber < NSAppKitVersionNumber10_14)
                        || (v as u32 > 4 && NSAppKitVersionNumber < NSAppKitVersionNumber10_11)
                    {
                        NSVisualEffectMaterial::AppearanceBased
                    } else {
                        NSVisualEffectMaterial(v as u64 as _)
                    };
                    vev.setMaterial(m);
                    vev.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
                    vev.setState(NSVisualEffectState::FollowsWindowActiveState);
                    vev.setAutoresizingMask(
                        NSAutoresizingMaskOptions::ViewWidthSizable
                            | NSAutoresizingMaskOptions::ViewHeightSizable,
                    );
                    view.addSubview_positioned_relativeTo(&vev, NSWindowOrderingMode::Below, None);
                    if let Some(vv) = self.vibrancy_view.replace(vev) {
                        vv.removeFromSuperview();
                    }
                } else if let Some(vv) = self.vibrancy_view.take() {
                    vv.removeFromSuperview();
                }
                Ok(())
            })
            .flatten()
        }
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
        BorrowedWindow::app_kit(&self.wnd)
    }
}

impl AsContainer for Window {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::app_kit(&self.content_view)
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            NSDistributedNotificationCenter::defaultCenter().removeObserver(&self.delegate);
        }
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
    #[name = "WinioWindowDelegate"]
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

        #[unsafe(method(userDefaultsDidChange))]
        unsafe fn userDefaultsDidChange(&self) {
            self.ivars().defaults_change.signal::<GlobalRuntime>(());
        }
    }

    unsafe impl NSObjectProtocol for WindowDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSWindowDelegate for WindowDelegate {
        #[unsafe(method(windowDidResize:))]
        unsafe fn windowDidResize(&self, _notification: &NSNotification) {
            self.ivars().did_resize.signal::<GlobalRuntime>(());
        }

        #[unsafe(method(windowDidMove:))]
        unsafe fn windowDidMove(&self, _notification: &NSNotification) {
            self.ivars().did_move.signal::<GlobalRuntime>(());
        }

        #[unsafe(method(windowShouldClose:))]
        unsafe fn windowShouldClose(&self, _sender: &NSWindow) -> bool {
            self.ivars().should_close.signal::<GlobalRuntime>(())
        }
    }
}

impl WindowDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}

/// <https://developer.apple.com/documentation/appkit/nsvisualeffectview/material>
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Vibrancy {
    #[deprecated(
        note = "A default material appropriate for the view's effectiveAppearance.  You should \
                instead choose an appropriate semantic material."
    )]
    AppearanceBased     = 0,
    #[deprecated(note = "Use a semantic material instead.")]
    Light               = 1,
    #[deprecated(note = "Use a semantic material instead.")]
    Dark                = 2,
    #[deprecated(note = "Use a semantic material instead.")]
    MediumLight         = 8,
    #[deprecated(note = "Use a semantic material instead.")]
    UltraDark           = 9,

    /// macOS 10.10+
    Titlebar            = 3,
    /// macOS 10.10+
    Selection           = 4,

    /// macOS 10.11+
    Menu                = 5,
    /// macOS 10.11+
    Popover             = 6,
    /// macOS 10.11+
    Sidebar             = 7,

    /// macOS 10.14+
    HeaderView          = 10,
    /// macOS 10.14+
    Sheet               = 11,
    /// macOS 10.14+
    WindowBackground    = 12,
    /// macOS 10.14+
    HudWindow           = 13,
    /// macOS 10.14+
    FullScreenUI        = 15,
    /// macOS 10.14+
    Tooltip             = 17,
    /// macOS 10.14+
    ContentBackground   = 18,
    /// macOS 10.14+
    UnderWindowBackground = 21,
    /// macOS 10.14+
    UnderPageBackground = 22,
}

#[derive(Debug)]
pub(crate) struct Widget {
    parent: Weak<NSView>,
    view: Retained<NSView>,
}

impl Widget {
    pub fn from_nsview(parent: impl AsContainer, view: Retained<NSView>) -> Result<Self> {
        let mut this = catch(|| {
            let parent = parent.as_container().as_app_kit();
            parent.addSubview(&view);
            Self {
                parent: Weak::from_retained(parent),
                view,
            }
        })?;
        this.set_loc(Point::zero())?;
        Ok(this)
    }

    pub fn parent(&self) -> Result<Retained<NSView>> {
        catch(|| self.parent.load())?.ok_or(Error::NullPointer)
    }

    pub fn is_visible(&self) -> Result<bool> {
        catch(|| !self.view.isHidden())
    }

    pub fn set_visible(&mut self, v: bool) -> Result<()> {
        catch(|| self.view.setHidden(!v))
    }

    pub fn is_enabled(&self) -> Result<bool> {
        catch(|| {
            self.view
                .downcast_ref::<NSControl>()
                .map(|c| c.isEnabled())
                .unwrap_or(true)
        })
    }

    pub fn set_enabled(&mut self, v: bool) -> Result<()> {
        catch(|| {
            if let Some(c) = self.view.downcast_ref::<NSControl>() {
                c.setEnabled(v);
            }
        })
    }

    pub fn preferred_size(&self) -> Result<Size> {
        catch(|| {
            let s = self.view.fittingSize();
            if s != NSSize::ZERO {
                return from_cgsize(s);
            }
            self.view
                .downcast_ref::<NSControl>()
                .map(|c| from_cgsize(c.sizeThatFits(NSSize::ZERO)))
                .unwrap_or_default()
        })
    }

    pub fn loc(&self) -> Result<Point> {
        catch(|| {
            let frame = self.view.frame();
            let screen_frame = self.parent()?.frame();
            Ok(transform_cgrect(from_cgsize(screen_frame.size), frame).origin)
        })
        .flatten()
    }

    pub fn set_loc(&mut self, p: Point) -> Result<()> {
        catch(|| {
            let frame = self.view.frame();
            let screen_frame = self.parent()?.frame();
            let frame = transform_rect(
                from_cgsize(screen_frame.size),
                Rect::new(p, from_cgsize(frame.size)),
            );
            self.view.setFrame(frame);
            Ok(())
        })
        .flatten()
    }

    pub fn size(&self) -> Result<Size> {
        catch(|| from_cgsize(self.view.frame().size))
    }

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        catch(|| {
            let mut frame = self.view.frame();
            let ydiff = v.height - frame.size.height;
            frame.size = to_cgsize(v);
            frame.origin.y -= ydiff;
            self.view.setFrame(frame);
        })
    }

    pub fn text(&self) -> Result<String> {
        catch(|| {
            self.view
                .downcast_ref::<NSControl>()
                .map(|c| from_nsstring(&c.stringValue()))
                .unwrap_or_default()
        })
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        catch(|| {
            if let Some(c) = self.view.downcast_ref::<NSControl>() {
                c.setStringValue(&NSString::from_str(s.as_ref()));
            }
        })
    }

    pub fn tooltip(&self) -> Result<String> {
        catch(|| {
            self.view
                .toolTip()
                .map(|s| from_nsstring(&s))
                .unwrap_or_default()
        })
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()> {
        catch(|| {
            let s = s.as_ref();
            let s = if s.is_empty() {
                None
            } else {
                Some(NSString::from_str(s))
            };
            self.view.setToolTip(s.as_deref());
        })
    }
}

impl Drop for Widget {
    fn drop(&mut self) {
        self.view.removeFromSuperview();
    }
}

impl AsWidget for Widget {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        BorrowedWidget::app_kit(&self.view)
    }
}

impl AsContainer for Widget {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::app_kit(&self.view)
    }
}

#[derive(Debug)]
pub struct View {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl View {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        unsafe {
            catch(|| {
                let parent = parent.as_container();
                let mtm = parent.as_app_kit().mtm();

                let view = NSView::new(mtm);
                let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

                Ok(Self { handle })
            })
            .flatten()
        }
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
