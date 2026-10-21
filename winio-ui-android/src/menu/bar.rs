use jni::{
    Env,
    refs::{Global, Reference},
};
use winio_handle::AsWindow;

use crate::{
    Error, Result, current_activity,
    java::{
        android::{
            view::{Menu as JMenu, gravity, visibility},
            widget::{FrameLayout, FrameLayoutLayoutParams},
        },
        custom::Activity,
        material::MaterialToolbar,
    },
    vm_exec,
    widgets::MENU_BAR_VIEW_ID,
};

#[derive(Debug)]
pub struct MenuBar {
    toolbar: Global<MaterialToolbar<'static>>,
    menu: Global<JMenu<'static>>,
    strip: Global<FrameLayout<'static>>,
}

impl MenuBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        vm_exec(|env| {
            let strip = menu_bar_view(env, &parent)?;
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

    pub(crate) fn menu(&self) -> &Global<JMenu<'static>> {
        &self.menu
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

fn menu_bar_view(
    env: &mut Env<'_>,
    parent: &impl AsWindow,
) -> Result<Global<FrameLayout<'static>>> {
    let window = env.new_local_ref(parent.as_window().to_android())?;
    let activity = unsafe { Activity::from_raw(env, window.into_raw()) };
    let strip = activity.find_view_by_id(env, MENU_BAR_VIEW_ID)?;
    if strip.as_raw().is_null() {
        return Err(Error::NoApp);
    }
    let strip = unsafe { FrameLayout::from_raw(env, strip.into_raw()) };
    Ok(env.new_global_ref(&strip)?)
}
