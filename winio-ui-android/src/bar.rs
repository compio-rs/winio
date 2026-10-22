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
    widgets::{MENU_BAR_VIEW_ID, TOOL_BAR_VIEW_ID, notify_layout_changed},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BarKind {
    MenuBar,
    ToolBar,
}

impl BarKind {
    fn view_id(self) -> i32 {
        match self {
            Self::MenuBar => MENU_BAR_VIEW_ID,
            Self::ToolBar => TOOL_BAR_VIEW_ID,
        }
    }

    fn exists_error(self) -> Error {
        match self {
            Self::MenuBar => Error::MenuBarExists,
            Self::ToolBar => Error::ToolBarExists,
        }
    }

    fn show_on_create(self) -> bool {
        matches!(self, Self::ToolBar)
    }

    fn hide_on_drop(self) -> bool {
        matches!(self, Self::ToolBar)
    }

    fn notify_layout(self) -> bool {
        matches!(self, Self::ToolBar)
    }
}

#[derive(Debug)]
pub(crate) struct BarBase {
    kind: BarKind,
    toolbar: Global<MaterialToolbar<'static>>,
    menu: Global<JMenu<'static>>,
    strip: Global<FrameLayout<'static>>,
}

impl BarBase {
    pub(crate) fn new(parent: impl AsWindow, kind: BarKind) -> Result<Self> {
        let this = vm_exec(|env| {
            let strip = bar_view(env, &parent, kind.view_id())?;
            if strip.as_view_group().get_child_count(env)? > 0 {
                return Err(kind.exists_error());
            }

            let act = current_activity(env)?;
            let toolbar = MaterialToolbar::new(env, &act)?;
            let params = FrameLayoutLayoutParams::new(env, -1, -2)?;
            params.set_gravity(env, gravity::LEFT | gravity::TOP)?;
            toolbar.as_view().set_layout_params(env, &params)?;
            let toolbar_menu = toolbar.get_menu(env)?;
            let menu = env.new_global_ref(&toolbar_menu)?;
            strip.as_view_group().add_view(env, toolbar.as_view())?;
            if kind.show_on_create() {
                strip.as_view().set_visibility(env, visibility::VISIBLE)?;
            }

            Ok(Self {
                kind,
                toolbar: env.new_global_ref(&toolbar)?,
                menu,
                strip,
            })
        })?;
        if kind.notify_layout() {
            notify_layout_changed();
        }
        Ok(this)
    }

    pub(crate) fn is_visible(&self) -> Result<bool> {
        vm_exec(|env| Ok(self.strip.as_view().get_visibility(env)? == visibility::VISIBLE))
    }

    pub(crate) fn set_visible(&mut self, visible: bool) -> Result<()> {
        vm_exec(|env| {
            self.strip.as_view().set_visibility(
                env,
                if visible {
                    visibility::VISIBLE
                } else {
                    visibility::GONE
                },
            )?;
            Result::Ok(())
        })?;
        if self.kind.notify_layout() {
            notify_layout_changed();
        }
        Ok(())
    }

    pub(crate) fn menu(&self) -> &Global<JMenu<'static>> {
        &self.menu
    }
}

impl Drop for BarBase {
    fn drop(&mut self) {
        let kind = self.kind;
        let _ = vm_exec(|env| -> Result<()> {
            self.strip
                .as_view_group()
                .remove_view(env, self.toolbar.as_view())?;
            if kind.hide_on_drop() {
                self.strip.as_view().set_visibility(env, visibility::GONE)?;
            }
            Ok(())
        });
        if kind.notify_layout() {
            notify_layout_changed();
        }
    }
}

fn bar_view(
    env: &mut Env<'_>,
    parent: &impl AsWindow,
    id: i32,
) -> Result<Global<FrameLayout<'static>>> {
    let window = env.new_local_ref(parent.as_window().to_android())?;
    let activity = unsafe { Activity::from_raw(env, window.into_raw()) };
    let strip = activity.find_view_by_id(env, id)?;
    if strip.as_raw().is_null() {
        return Err(Error::NoApp);
    }
    let strip = unsafe { FrameLayout::from_raw(env, strip.into_raw()) };
    Ok(env.new_global_ref(&strip)?)
}
