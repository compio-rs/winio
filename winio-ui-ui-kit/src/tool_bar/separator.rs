use objc2::MainThreadMarker;
use objc2_ui_kit::{NSLayoutConstraint, UIBarButtonItem, UIColor, UIView};

use crate::{
    Error, Result, catch,
    tool_bar::{ToolBar, ToolBarItem},
};

/// A vertical separator in a [`ToolBar`].
#[derive(Debug)]
pub struct ToolBarSeparator {
    item: ToolBarItem,
}

impl ToolBarSeparator {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;

        catch(|| {
            let line = UIView::new(mtm);
            line.setTranslatesAutoresizingMaskIntoConstraints(false);
            line.setUserInteractionEnabled(false);
            line.setBackgroundColor(Some(&UIColor::separatorColor()));
            let width = line.widthAnchor().constraintEqualToConstant(1.0);
            let height = line.heightAnchor().constraintEqualToConstant(24.0);
            NSLayoutConstraint::activateConstraints(
                &objc2_foundation::NSArray::from_retained_slice(&[width, height]),
                mtm,
            );

            let bar_item = UIBarButtonItem::initWithCustomView(mtm.alloc(), &line);
            let item = ToolBarItem::new(parent, bar_item)?;
            Ok(Self { item })
        })
        .flatten()
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarSeparator, item);
