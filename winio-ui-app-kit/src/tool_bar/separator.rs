use objc2_app_kit::{NSBox, NSBoxType, NSColor, NSTitlePosition};
use objc2_foundation::NSSize;

use crate::{
    Result, catch,
    tool_bar::{ToolBar, ToolBarItem},
};

/// A vertical separator in a [`ToolBar`].
#[derive(Debug)]
pub struct ToolBarSeparator {
    item: ToolBarItem,
}

impl ToolBarSeparator {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let mtm = parent.data().mtm();

        catch(|| {
            let separator = NSBox::new(mtm);
            separator.setBoxType(NSBoxType::Custom);
            separator.setTitlePosition(NSTitlePosition::NoTitle);
            separator.setBorderWidth(0.0);
            separator.setTransparent(false);
            let color = NSColor::separatorColor();
            separator.setFillColor(&color);
            separator.setFrameSize(NSSize::new(1.0, 20.0));

            let item = ToolBarItem::new(parent, &separator, "")?;
            Ok(Self { item })
        })
        .flatten()
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarSeparator, item);
