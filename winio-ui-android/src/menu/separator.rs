use jni::objects::JObject;
use winio_handle::{AsMenuContainer, AsMenuItem, BorrowedMenuItem};

use crate::Result;

#[derive(Debug)]
pub struct SeparatorMenuItem {
    item: JObject<'static>,
}

impl SeparatorMenuItem {
    pub fn new(_parent: impl AsMenuContainer) -> Result<Self> {
        Ok(Self {
            item: JObject::null(),
        })
    }
}

impl AsMenuItem for SeparatorMenuItem {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        unsafe { BorrowedMenuItem::android(&self.item) }
    }
}
