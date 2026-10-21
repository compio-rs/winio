use objc2::rc::Retained;
use objc2_foundation::{MainThreadMarker, NSArray};
use objc2_ui_kit::{NSLayoutConstraint, UILayoutConstraintAxis, UIScrollView, UIStackView};
use winio_handle::AsWindow;

use crate::{Error, Result, catch, widgets::MenuBarHost};

#[derive(Debug)]
pub struct MenuBar {
    stack: Retained<UIStackView>,
    host: MenuBarHost,
    scroll: Retained<UIScrollView>,
    grow: Retained<NSLayoutConstraint>,
}

impl MenuBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
        catch(|| {
            let host = MenuBarHost::of(parent.as_window().as_ui_kit())?;
            if !host.view.subviews().is_empty() {
                return Err(Error::MenuBarExists);
            }

            let scroll = UIScrollView::new(mtm);
            scroll.setTranslatesAutoresizingMaskIntoConstraints(false);
            scroll.setShowsHorizontalScrollIndicator(false);
            scroll.setShowsVerticalScrollIndicator(false);
            host.view.addSubview(&scroll);

            let stack = UIStackView::new(mtm);
            stack.setTranslatesAutoresizingMaskIntoConstraints(false);
            stack.setAxis(UILayoutConstraintAxis::Horizontal);
            stack.setSpacing(4.0);
            scroll.addSubview(&stack);

            let content = scroll.contentLayoutGuide();
            let a1 = stack
                .leadingAnchor()
                .constraintEqualToAnchor(&content.leadingAnchor());
            let a2 = stack
                .trailingAnchor()
                .constraintEqualToAnchor(&content.trailingAnchor());
            let a3 = stack
                .topAnchor()
                .constraintEqualToAnchor(&content.topAnchor());
            let a4 = stack
                .bottomAnchor()
                .constraintEqualToAnchor(&content.bottomAnchor());
            let b1 = scroll
                .leadingAnchor()
                .constraintEqualToAnchor(&host.view.leadingAnchor());
            let b2 = scroll
                .trailingAnchor()
                .constraintEqualToAnchor(&host.view.trailingAnchor());
            let b3 = scroll
                .topAnchor()
                .constraintEqualToAnchor(&host.view.topAnchor());
            let b4 = scroll
                .bottomAnchor()
                .constraintEqualToAnchor(&host.view.bottomAnchor());
            NSLayoutConstraint::activateConstraints(
                &NSArray::from_retained_slice(&[a1, a2, a3, a4, b1, b2, b3, b4]),
                mtm,
            );

            // The strip takes the natural height of the buttons, so it follows
            // the text size of the user.
            let grow = host
                .view
                .heightAnchor()
                .constraintEqualToAnchor(&stack.heightAnchor());
            host.zero_height.setActive(false);
            grow.setActive(true);

            Ok(Self {
                stack,
                host,
                scroll,
                grow,
            })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool> {
        catch(|| !self.host.view.isHidden())
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        catch(|| {
            self.host.view.setHidden(!visible);
            self.grow.setActive(visible);
            self.host.zero_height.setActive(!visible);
        })
    }

    pub(crate) fn stack(&self) -> &Retained<UIStackView> {
        &self.stack
    }
}

impl Drop for MenuBar {
    fn drop(&mut self) {
        self.grow.setActive(false);
        self.host.zero_height.setActive(true);
        self.scroll.removeFromSuperview();
    }
}
