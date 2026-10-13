use std::rc::Rc;

use gtk4::{
    gio::{self, prelude::*},
    glib::{SignalHandlerId, VariantTy},
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsMenuContainer;

use crate::{
    GlobalRuntime, Result,
    menu::{ACTION_PREFIX, ACTIONS, ItemBase, alloc_action_name},
};

#[derive(Debug)]
pub struct RadioMenuItem {
    item: ItemBase,
    action: gio::SimpleAction,
    on_click: Rc<Callback<()>>,
    handler: Option<SignalHandlerId>,
}

#[inherit_methods(from = "self.item")]
impl RadioMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().to_gtk();

        let action_name = alloc_action_name();
        let action = gio::SimpleAction::new_stateful(
            &action_name,
            Some(VariantTy::BOOLEAN),
            &false.to_variant(),
        );
        ACTIONS.with(|actions| actions.add_action(&action));
        let detailed_action = format!("{}.{}", ACTION_PREFIX, action_name);
        let item = ItemBase::new(
            parent,
            Some(&detailed_action),
            Some(&true.to_variant()),
            None,
        )?;

        let on_click = Rc::new(Callback::new());
        let handler = action.connect_activate({
            let action = action.clone();
            let on_click = on_click.clone();
            move |_, _| {
                action.set_state(&true.to_variant());
                on_click.signal::<GlobalRuntime>(());
            }
        });

        Ok(Self {
            item,
            action,
            on_click,
            handler: Some(handler),
        })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(self.action.is_enabled())
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        self.action.set_enabled(enabled);
        Ok(())
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool> {
        Ok(self
            .action
            .state()
            .and_then(|state| state.get::<bool>())
            .unwrap_or(false))
    }

    pub fn set_checked(&mut self, checked: bool) -> Result<()> {
        self.action.set_state(&checked.to_variant());
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

impl Drop for RadioMenuItem {
    fn drop(&mut self) {
        if let Some(handler) = self.handler.take() {
            self.action.disconnect(handler);
        }
    }
}

winio_handle::impl_as_menu_item!(RadioMenuItem, item);
