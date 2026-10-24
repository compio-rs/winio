use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{
    objects::JObject,
    refs::{Global, LoaderContext, Reference},
};
use jni_min_helper::DynamicProxy;
use winio_handle::{AsContainer, AsMenuContainer, BorrowedMenuContainer, impl_as_widget};
use winio_primitive::{Point, Size};

use crate::{
    BaseWidget, Image, JCharSequenceExt, Result, current_activity,
    java::{
        android::{
            content::res::ColorStateList,
            graphics::drawable::Drawable,
            view::{Menu as JMenu, ViewOnClickListener},
            widget::{Button as AButton, PopupMenu, TextView as ATextView},
        },
        material::MaterialButton,
    },
    vm_exec,
};

#[derive(Debug)]
pub struct MenuButton {
    inner: BaseWidget<MaterialButton<'static>>,
    #[allow(dead_code)]
    popup: Arc<Global<PopupMenu<'static>>>,
    menu: Global<JMenu<'static>>,
    #[allow(dead_code)]
    click_proxy: DynamicProxy,
}

#[inherit_methods(from = "self.inner")]
impl MenuButton {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = MaterialButton::new(env, &act)?;
            let popup = PopupMenu::new(env, &act, &widget)?;
            let menu = popup.get_menu(env)?;
            let menu = env.new_global_ref(&menu)?;
            let popup = Arc::new(env.new_global_ref(&popup)?);

            let popup_for_click = popup.clone();
            let click_proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [ViewOnClickListener::class_name()],
                move |env, _method, _args| {
                    popup_for_click.show(env)?;
                    Ok(JObject::null())
                },
            )?;
            AsRef::<AButton>::as_ref(&widget).set_on_click_listener(env, &click_proxy)?;

            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            Ok(Self {
                inner,
                popup,
                menu,
                click_proxy,
            })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    fn as_text_view(&self) -> &ATextView<'static> {
        self.inner.as_ref()
    }

    pub fn text(&self) -> Result<String> {
        vm_exec(move |env| Ok(self.as_text_view().get_text(env)?.try_to_string(env)?))
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        vm_exec(move |env| {
            let text = env.new_string(text.as_ref())?;
            self.as_text_view().set_text(env, &text)?;
            Ok(())
        })
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        vm_exec(|env| {
            self.inner.set_icon_tint(env, ColorStateList::null())?;
            if let Some(icon) = icon {
                let drawable = icon.drawable(env)?;
                self.inner.set_icon(env, &drawable)?;
            } else {
                self.inner.set_icon(env, Drawable::null())?;
            }
            Ok(())
        })
    }

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

impl AsMenuContainer for MenuButton {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        unsafe { BorrowedMenuContainer::android(&self.menu) }
    }
}

impl_as_widget!(MenuButton, inner);
