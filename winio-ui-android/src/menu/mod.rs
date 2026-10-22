use std::{
    cell::Cell,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
    },
};

use jni::{
    Env,
    refs::{Global, LoaderContext, Reference},
};
use jni_min_helper::{DynamicProxy, JBoolean};
use winio_callback::SyncCallback;
use winio_handle::{AsMenuContainer, AsMenuItem, BorrowedMenuContainer, BorrowedMenuItem};

use crate::{
    Image, JCharSequenceExt, Result,
    java::android::{
        graphics::drawable::Drawable,
        view::{
            Menu as JMenu, MenuItem as JMenuItem, MenuItemOnMenuItemClickListener,
            SubMenu as JSubMenu, menu, menu_item,
        },
    },
    runtime::AndroidRuntime,
};

mod bar;
pub use bar::*;

mod bar_item;
pub use bar_item::*;

mod sub_menu_button;
pub(crate) use sub_menu_button::*;

mod item;
pub use item::*;

mod popup;
pub use popup::*;

mod check;
pub use check::*;

mod radio;
pub use radio::*;

mod separator;
pub use separator::*;

fn next_id() -> i32 {
    static NEXT: AtomicI32 = AtomicI32::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

fn container_menu<'local>(
    env: &mut Env<'local>,
    container: BorrowedMenuContainer<'_>,
) -> Result<JMenu<'local>> {
    let menu = env.new_local_ref(container.to_android())?;
    Ok(unsafe { JMenu::from_raw(env, menu.into_raw()) })
}

fn click_listener(env: &mut Env<'_>, on_click: &Arc<SyncCallback>) -> Result<DynamicProxy> {
    let on_click = on_click.clone();
    Ok(DynamicProxy::build(
        env,
        &LoaderContext::None,
        [MenuItemOnMenuItemClickListener::class_name()],
        move |env, _method, _args| {
            on_click.signal::<AndroidRuntime>(());
            Ok(JBoolean::new(env, true)?.into())
        },
    )?)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LeafKind {
    Plain,
    Check,
    Radio,
}

#[derive(Debug)]
pub(crate) struct LeafItem {
    menu: Global<JMenu<'static>>,
    item: Global<JMenuItem<'static>>,
    id: i32,
    group: Option<i32>,
    checked: Cell<bool>,
    on_click: Arc<SyncCallback>,
    #[allow(dead_code)]
    click_proxy: DynamicProxy,
}

impl LeafItem {
    pub fn new(
        env: &mut Env<'_>,
        parent: BorrowedMenuContainer<'_>,
        kind: LeafKind,
    ) -> Result<Self> {
        let menu = container_menu(env, parent)?;
        let on_click = Arc::new(SyncCallback::new());
        let click_proxy = click_listener(env, &on_click)?;
        let id = next_id();
        let group = match kind {
            LeafKind::Radio => next_id(),
            LeafKind::Plain | LeafKind::Check => menu::NONE,
        };
        let title = env.new_string("")?;
        let item = menu.add(env, group, id, id, title)?;
        match kind {
            LeafKind::Plain => {}
            LeafKind::Check => {
                item.set_checkable(env, true)?;
            }
            LeafKind::Radio => {
                menu.set_group_checkable(env, group, true, true)?;
            }
        }
        item.set_on_menu_item_click_listener(env, &click_proxy)?;
        Ok(Self {
            menu: env.new_global_ref(&menu)?,
            item: env.new_global_ref(&item)?,
            id,
            checked: Cell::new(false),
            group: (kind == LeafKind::Radio).then_some(group),
            on_click,
            click_proxy,
        })
    }

    pub fn text(&self, env: &mut Env<'_>) -> Result<String> {
        Ok(self.item.get_title(env)?.try_to_string(env)?)
    }

    pub fn set_text(&self, env: &mut Env<'_>, s: impl AsRef<str>) -> Result<()> {
        let title = env.new_string(s.as_ref())?;
        self.item.set_title(env, title)?;
        Ok(())
    }

    pub fn set_icon(&self, env: &mut Env<'_>, icon: Option<&Image>) -> Result<()> {
        match icon {
            Some(icon) => {
                let drawable = icon.drawable(env)?;
                self.item.set_icon(env, drawable)?;
            }
            None => {
                self.item.set_icon(env, Drawable::null())?;
            }
        }
        Ok(())
    }

    pub fn is_enabled(&self, env: &mut Env<'_>) -> Result<bool> {
        Ok(self.item.is_enabled(env)?)
    }

    pub fn set_enabled(&self, env: &mut Env<'_>, enabled: bool) -> Result<()> {
        self.item.set_enabled(env, enabled)?;
        Ok(())
    }

    pub fn is_checked(&self, _env: &mut Env<'_>) -> Result<bool> {
        Ok(self.checked.get())
    }

    pub fn set_checked(&self, env: &mut Env<'_>, checked: bool) -> Result<()> {
        self.checked.set(checked);
        match self.group {
            // `MenuItem.setChecked(false)` of the item of an exclusive group
            // checks it instead: the framework throws the argument away and
            // asks the menu to check the item and uncheck its group. So the
            // exclusivity is dropped for that single call and put back right
            // after. The message is drawn when the item is bound, from the flag
            // that is already restored by then.
            Some(group) if !checked => {
                self.menu.set_group_checkable(env, group, true, false)?;
                self.item.set_checked(env, false)?;
                self.menu.set_group_checkable(env, group, true, true)?;
            }
            _ => {
                self.item.set_checked(env, checked)?;
            }
        }
        Ok(())
    }

    pub fn toggle_checked(&self, env: &mut Env<'_>) -> Result<()> {
        let checked = self.item.is_checked(env)?;
        self.set_checked(env, !checked)
    }

    pub fn set_show_as_action(&self, env: &mut Env<'_>) -> Result<()> {
        self.item.set_show_as_action(
            env,
            menu_item::SHOW_AS_ACTION_ALWAYS | menu_item::SHOW_AS_ACTION_WITH_TEXT,
        )?;
        Ok(())
    }

    pub(crate) fn item(&self) -> &Global<JMenuItem<'static>> {
        &self.item
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await;
    }
}

impl Drop for LeafItem {
    fn drop(&mut self) {
        let id = self.id;
        let _ = crate::vm_exec(move |env| -> Result<()> {
            self.menu.remove_item(env, id)?;
            Ok(())
        });
    }
}

impl AsMenuItem for LeafItem {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        // SAFETY: the item is a valid `android.view.MenuItem`.
        unsafe { BorrowedMenuItem::android(&self.item) }
    }
}

#[derive(Debug)]
pub(crate) struct SubMenuItem {
    menu: Global<JMenu<'static>>,
    submenu: Global<JSubMenu<'static>>,
    item: Global<JMenuItem<'static>>,
    id: i32,
}

impl SubMenuItem {
    /// Add a submenu to `menu`.
    pub fn new(env: &mut Env<'_>, menu: &JMenu<'_>) -> Result<Self> {
        let id = next_id();
        let title = env.new_string("")?;
        let submenu = menu.add_sub_menu(env, menu::NONE, id, id, title)?;
        let item = submenu.get_item(env)?;
        Ok(Self {
            menu: env.new_global_ref(menu)?,
            submenu: env.new_global_ref(&submenu)?,
            item: env.new_global_ref(&item)?,
            id,
        })
    }

    pub fn text(&self, env: &mut Env<'_>) -> Result<String> {
        Ok(self.item.get_title(env)?.try_to_string(env)?)
    }

    pub fn set_text(&self, env: &mut Env<'_>, s: impl AsRef<str>) -> Result<()> {
        let title = env.new_string(s.as_ref())?;
        self.item.set_title(env, title)?;
        Ok(())
    }

    pub fn set_icon(&self, env: &mut Env<'_>, icon: Option<&Image>) -> Result<()> {
        match icon {
            Some(icon) => {
                let drawable = icon.drawable(env)?;
                self.item.set_icon(env, drawable)?;
            }
            None => {
                self.item.set_icon(env, Drawable::null())?;
            }
        }
        Ok(())
    }

    pub fn is_enabled(&self, env: &mut Env<'_>) -> Result<bool> {
        Ok(self.item.is_enabled(env)?)
    }

    pub fn set_enabled(&self, env: &mut Env<'_>, enabled: bool) -> Result<()> {
        self.item.set_enabled(env, enabled)?;
        Ok(())
    }

    /// Show this submenu as an action of the toolbar.
    pub fn set_show_as_action(&self, env: &mut Env<'_>) -> Result<()> {
        self.item.set_show_as_action(
            env,
            menu_item::SHOW_AS_ACTION_ALWAYS | menu_item::SHOW_AS_ACTION_WITH_TEXT,
        )?;
        Ok(())
    }

    pub(crate) fn item(&self) -> &Global<JMenuItem<'static>> {
        &self.item
    }
}

impl Drop for SubMenuItem {
    fn drop(&mut self) {
        let id = self.id;
        let _ = crate::vm_exec(move |env| -> Result<()> {
            self.menu.remove_item(env, id)?;
            Ok(())
        });
    }
}

impl AsMenuContainer for SubMenuItem {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        // SAFETY: the submenu is a valid `android.view.Menu`.
        unsafe { BorrowedMenuContainer::android(&self.submenu) }
    }
}

impl AsMenuItem for SubMenuItem {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        // SAFETY: the item is a valid `android.view.MenuItem`.
        unsafe { BorrowedMenuItem::android(&self.item) }
    }
}
