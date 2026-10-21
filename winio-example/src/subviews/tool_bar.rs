use winio::prelude::*;

use crate::{Error, Result};

pub struct ExampleToolBar {
    tool_bar: Child<ToolBar>,
    open_file: Child<ToolBarButton>,
    combo: Child<ToolBarDropDownButton>,
    combo_push: Child<MenuItem>,
    combo_pop: Child<MenuItem>,
    combo_show: Child<MenuItem>,
    separator: Child<ToolBarSeparator>,
    show_password: Child<ToolBarCheckBox>,
    wgpu_separator: Child<ToolBarSeparator>,
    wgpu_option_group: Child<RadioGroup<ToolBarRadioButton>>,
    eye_icon: Image,
    eye_white_icon: Image,
    clockwise_icon: Image,
    clockwise_white_icon: Image,
    counter_clockwise_icon: Image,
    counter_clockwise_white_icon: Image,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ExampleToolBarEvent {
    Push,
    Pop,
    Show,
    OpenFile,
    WgpuOption(bool),
}

#[derive(Debug)]
pub enum ExampleToolBarMessage {
    Push,
    Pop,
    Show,
    OpenFile,
    SetShowPassword(bool),
    SetWgpuOption(Option<usize>),
    Redraw,
    WgpuOption(Option<usize>),
}

impl Component for ExampleToolBar {
    type Error = Error;
    type Event = ExampleToolBarEvent;
    type Init<'a> = BorrowedWindow<'a>;
    type Message = ExampleToolBarMessage;

    async fn init(init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        let eye_icon = crate::icons::eye()?;
        let eye_white_icon = crate::icons::eye_white()?;
        let clockwise_icon = crate::icons::clockwise()?;
        let clockwise_white_icon = crate::icons::clockwise_white()?;
        let counter_clockwise_icon = crate::icons::counter_clockwise()?;
        let counter_clockwise_white_icon = crate::icons::counter_clockwise_white()?;
        let dark = ColorTheme::current()? == ColorTheme::Dark;
        let (eye, clockwise, counter_clockwise) = if dark {
            (
                &eye_white_icon,
                &clockwise_white_icon,
                &counter_clockwise_white_icon,
            )
        } else {
            (&eye_icon, &clockwise_icon, &counter_clockwise_icon)
        };

        init! {
            tool_bar: ToolBar = (init),
            open_file: ToolBarButton = (&*tool_bar) => {
                text: "Open...",
                icon: Some(&crate::icons::open()?),
                enabled: cfg!(feature = "compio-compat"),
            },
            combo: ToolBarDropDownButton = (&*tool_bar) => {
                text: "Combo",
            },
            combo_push: MenuItem = (&combo) => {
                text: "Push",
            },
            combo_pop: MenuItem = (&combo) => {
                text: "Pop",
            },
            combo_show: MenuItem = (&combo) => {
                text: "Show",
            },
            separator: ToolBarSeparator = (&*tool_bar),
            show_password: ToolBarCheckBox = (&*tool_bar) => {
                icon: Some(eye),
            },
            wgpu_separator: ToolBarSeparator = (&*tool_bar),
            wgpu_clockwise: ToolBarRadioButton = (&*tool_bar) => {
                icon: Some(clockwise),
                enabled: cfg!(feature = "wgpu"),
            },
            wgpu_counter_clockwise: ToolBarRadioButton = (&*tool_bar) => {
                icon: Some(counter_clockwise),
                enabled: cfg!(feature = "wgpu"),
            },
            wgpu_option_group: RadioGroup<ToolBarRadioButton> = ([wgpu_clockwise, wgpu_counter_clockwise]) => {
                selection: 0,
            },
        }

        show_password
            .checked_prop()?
            .bind(sender, ExampleToolBarMessage::SetShowPassword);
        wgpu_option_group
            .selection_prop()?
            .bind(sender, ExampleToolBarMessage::WgpuOption);

        Ok(Self {
            tool_bar,
            open_file,
            combo,
            combo_push,
            combo_pop,
            combo_show,
            separator,
            show_password,
            wgpu_separator,
            wgpu_option_group,
            eye_icon,
            eye_white_icon,
            clockwise_icon,
            clockwise_white_icon,
            counter_clockwise_icon,
            counter_clockwise_white_icon,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender,
            self.tool_bar => {},
            self.open_file => {
                ToolBarButtonEvent::Click => ExampleToolBarMessage::OpenFile,
            },
            self.combo => {},
            self.combo_push => {
                MenuItemEvent::Click => ExampleToolBarMessage::Push,
            },
            self.combo_pop => {
                MenuItemEvent::Click => ExampleToolBarMessage::Pop,
            },
            self.combo_show => {
                MenuItemEvent::Click => ExampleToolBarMessage::Show,
            },
            self.separator => {},
            self.show_password => {},
            self.wgpu_separator => {},
            self.wgpu_option_group => {},
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(
            self.tool_bar,
            self.open_file,
            self.combo,
            self.combo_push,
            self.combo_pop,
            self.combo_show,
            self.separator,
            self.show_password,
            self.wgpu_separator,
            self.wgpu_option_group
        )
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ExampleToolBarMessage::Push => {
                sender.output(ExampleToolBarEvent::Push);
                Ok(false)
            }
            ExampleToolBarMessage::Pop => {
                sender.output(ExampleToolBarEvent::Pop);
                Ok(false)
            }
            ExampleToolBarMessage::Show => {
                sender.output(ExampleToolBarEvent::Show);
                Ok(false)
            }
            ExampleToolBarMessage::OpenFile => {
                sender.output(ExampleToolBarEvent::OpenFile);
                Ok(false)
            }
            ExampleToolBarMessage::SetShowPassword(show) => {
                self.show_password.set_checked(show)?;
                Ok(false)
            }
            ExampleToolBarMessage::SetWgpuOption(selection) => {
                if let Some(index) = selection {
                    self.wgpu_option_group.set_selection(index)?;
                }
                Ok(false)
            }
            ExampleToolBarMessage::Redraw => {
                let dark = ColorTheme::current()? == ColorTheme::Dark;
                let (eye, clockwise, counter_clockwise) = if dark {
                    (
                        &self.eye_white_icon,
                        &self.clockwise_white_icon,
                        &self.counter_clockwise_white_icon,
                    )
                } else {
                    (
                        &self.eye_icon,
                        &self.clockwise_icon,
                        &self.counter_clockwise_icon,
                    )
                };
                self.show_password.set_icon(Some(eye))?;
                if let Some(radio) = self.wgpu_option_group.get_mut(0) {
                    radio.set_icon(Some(clockwise))?;
                }
                if let Some(radio) = self.wgpu_option_group.get_mut(1) {
                    radio.set_icon(Some(counter_clockwise))?;
                }
                Ok(false)
            }
            ExampleToolBarMessage::WgpuOption(selection) => {
                sender.output(ExampleToolBarEvent::WgpuOption(selection == Some(0)));
                Ok(false)
            }
        }
    }
}

impl ExampleToolBar {
    /// The checked state of the "Show password" check box.
    pub fn show_password_prop(&mut self) -> Result<Prop<'_, bool>> {
        Ok(self.show_password.checked_prop()?)
    }

    /// The selected rotation direction.
    pub fn wgpu_option_prop(&mut self) -> Result<Prop<'_, Option<usize>>> {
        Ok(self.wgpu_option_group.selection_prop()?)
    }
}
