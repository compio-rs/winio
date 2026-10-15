use jni::refs::Global;
use winio_handle::{AsMenuBar, AsWindow, BorrowedMenuBar};

use crate::{
    Error, Result, current_activity,
    java::{
        android::{
            view::{Menu as JMenu, gravity, visibility},
            widget::{FrameLayout, FrameLayoutLayoutParams},
        },
        material::MaterialToolbar,
    },
    vm_exec,
    widgets::MENU_BAR_VIEW,
};

#[derive(Debug)]
pub struct MenuBar {
    toolbar: Global<MaterialToolbar<'static>>,
    menu: Global<JMenu<'static>>,
    strip: Global<FrameLayout<'static>>,
}

impl MenuBar {
    pub fn new(_parent: impl AsWindow) -> Result<Self> {
        vm_exec(|env| {
            let strip = MENU_BAR_VIEW
                .lock()
                .unwrap()
                .as_ref()
                .map(|strip| env.new_global_ref(strip))
                .transpose()?
                .ok_or(Error::NoApp)?;
            if strip.as_view_group().get_child_count(env)? > 0 {
                return Err(Error::MenuBarExists);
            }

            let act = current_activity(env)?;
            let toolbar = MaterialToolbar::new(env, &act)?;
            let params = FrameLayoutLayoutParams::new(env, -1, -2)?;
            params.set_gravity(env, gravity::LEFT | gravity::TOP)?;
            toolbar.as_view().set_layout_params(env, &params)?;
            let toolbar_menu = toolbar.get_menu(env)?;
            let menu = env.new_global_ref(&toolbar_menu)?;
            strip.as_view_group().add_view(env, toolbar.as_view())?;

            Ok(Self {
                toolbar: env.new_global_ref(&toolbar)?,
                menu,
                strip,
            })
        })
    }

    pub fn is_visible(&self) -> Result<bool> {
        vm_exec(|env| Ok(self.strip.as_view().get_visibility(env)? == visibility::VISIBLE))
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        vm_exec(|env| {
            self.strip.as_view().set_visibility(
                env,
                if visible {
                    visibility::VISIBLE
                } else {
                    visibility::GONE
                },
            )?;
            Ok(())
        })
    }
}

impl AsMenuBar for MenuBar {
    fn as_menu_bar(&self) -> BorrowedMenuBar<'_> {
        // SAFETY: the menu is a valid `android.view.Menu`.
        unsafe { BorrowedMenuBar::android(&self.menu) }
    }
}

impl Drop for MenuBar {
    fn drop(&mut self) {
        let _ = vm_exec(|env| -> Result<()> {
            self.strip
                .as_view_group()
                .remove_view(env, self.toolbar.as_view())?;
            Ok(())
        });
    }
}
