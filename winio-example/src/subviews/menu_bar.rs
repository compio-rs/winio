//! The example menu bar.

use winio::prelude::*;

use crate::{Error, Result};

pub struct ExampleMenuBar {
    menu_bar: Child<MenuBar>,
    file_item: Child<MenuBarItem>,
    open_file_item: Child<MenuItem>,
    widgets_item: Child<MenuBarItem>,
    combo_control_item: Child<PopupMenuItem>,
    push_item: Child<MenuItem>,
    pop_item: Child<MenuItem>,
    show_item: Child<MenuItem>,
    #[allow(dead_code)]
    sep_item: Child<SeparatorMenuItem>,
    show_password_item: Child<CheckMenuItem>,
}

#[derive(Debug)]
pub enum ExampleMenuBarEvent {
    Push,
    Pop,
    Show,
    ShowPassword(bool),
    OpenFile,
}

#[derive(Debug)]
pub enum ExampleMenuBarMessage {
    Push,
    Pop,
    Show,
    ChangeInputShowPassword,
    OpenFile,
}

impl Component for ExampleMenuBar {
    type Error = Error;
    type Event = ExampleMenuBarEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ExampleMenuBarMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            menu_bar: MenuBar = (init),
            file_item: MenuBarItem = (&menu_bar) => {
                text: "File",
            },
            open_file_item: MenuItem = (&file_item) => {
                text: "Open file...",
            },
            widgets_item: MenuBarItem = (&menu_bar) => {
                text: "Widgets",
            },
            combo_control_item: PopupMenuItem = (&widgets_item) => {
                text: "Combo control",
            },
            push_item: MenuItem = (&combo_control_item) => {
                text: "Push",
            },
            pop_item: MenuItem = (&combo_control_item) => {
                text: "Pop",
            },
            show_item: MenuItem = (&combo_control_item) => {
                text: "Show",
            },
            sep_item: SeparatorMenuItem = (&widgets_item),
            show_password_item: CheckMenuItem = (&widgets_item) => {
                text: "Show password",
            },
        }

        Ok(Self {
            menu_bar,
            file_item,
            open_file_item,
            widgets_item,
            combo_control_item,
            push_item,
            pop_item,
            show_item,
            sep_item,
            show_password_item,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender,
            self.menu_bar => {},
            self.widgets_item => {},
            self.combo_control_item => {},
            self.push_item => {
                MenuItemEvent::Click => ExampleMenuBarMessage::Push,
            },
            self.pop_item => {
                MenuItemEvent::Click => ExampleMenuBarMessage::Pop,
            },
            self.show_item => {
                MenuItemEvent::Click => ExampleMenuBarMessage::Show,
            },
            self.show_password_item => {
                CheckMenuItemEvent::Click => ExampleMenuBarMessage::ChangeInputShowPassword,
            },
            self.file_item => {},
            self.open_file_item => {
                MenuItemEvent::Click => ExampleMenuBarMessage::OpenFile,
            },
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(
            self.menu_bar,
            self.widgets_item,
            self.combo_control_item,
            self.push_item,
            self.pop_item,
            self.show_item,
            self.show_password_item,
            self.file_item,
            self.open_file_item
        )
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ExampleMenuBarMessage::Push => {
                sender.output(ExampleMenuBarEvent::Push);
                Ok(false)
            }
            ExampleMenuBarMessage::Pop => {
                sender.output(ExampleMenuBarEvent::Pop);
                Ok(false)
            }
            ExampleMenuBarMessage::Show => {
                sender.output(ExampleMenuBarEvent::Show);
                Ok(false)
            }
            ExampleMenuBarMessage::ChangeInputShowPassword => {
                sender.output(ExampleMenuBarEvent::ShowPassword(
                    self.show_password_item.is_checked()?,
                ));
                Ok(false)
            }
            ExampleMenuBarMessage::OpenFile => {
                sender.output(ExampleMenuBarEvent::OpenFile);
                Ok(false)
            }
        }
    }
}
