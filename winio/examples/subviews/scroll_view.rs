use std::ops::Deref;

use tuplex::IntoArray;
use winio::prelude::*;

use crate::{Error, Result};

pub struct ScrollViewPage {
    window: Child<TabViewItem>,
    scroll: Child<ScrollView>,
    radios: Vec<Child<RadioButton>>,
    add_btn: Child<Button>,
    del_btn: Child<Button>,
    show_btn: Child<Button>,
    selected: Option<usize>,
}

#[derive(Debug)]
pub enum ScrollViewPageEvent {
    ShowMessage(MessageBox),
}

#[derive(Debug)]
pub enum ScrollViewPageMessage {
    Noop,
    Add,
    Del,
    Show,
    Select(usize),
}

impl Component for ScrollViewPage {
    type Error = Error;
    type Event = ScrollViewPageEvent;
    type Init<'a> = &'a TabView;
    type Message = ScrollViewPageMessage;

    fn init(tabview: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: TabViewItem = (tabview) => {
                text: "ScrollView",
            },
            scroll: ScrollView = (&window) => {
                vscroll: true,
                hscroll: false,
            },
            add_btn: Button = (&window) => {
                text: "Add Radio",
            },
            del_btn: Button = (&window) => {
                text: "Delete Radio",
            },
            show_btn: Button = (&window) => {
                text: "Show Selected",
            },
        }

        let radios = Vec::new();

        Ok(Self {
            window,
            scroll,
            radios,
            add_btn,
            del_btn,
            show_btn,
            selected: None,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let radios = self.radios.iter_mut().map(|r| &mut **r).collect::<Vec<_>>();
        let mut radio_group = RadioButtonGroup::new(radios);
        start! {
            sender, default: ScrollViewPageMessage::Noop,
            self.add_btn => {
                ButtonEvent::Click => ScrollViewPageMessage::Add,
            },
            self.del_btn => {
                ButtonEvent::Click => ScrollViewPageMessage::Del,
            },
            self.show_btn => {
                ButtonEvent::Click => ScrollViewPageMessage::Show,
            },
            self.scroll => {},
            radio_group => {
                |i| Some(ScrollViewPageMessage::Select(i))
            }
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        Ok(futures_util::future::try_join5(
            self.window.update(),
            self.scroll.update(),
            self.add_btn.update(),
            self.del_btn.update(),
            self.show_btn.update(),
        )
        .await?
        .into_array()
        .into_iter()
        .any(|b| b))
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ScrollViewPageMessage::Noop => Ok(false),
            ScrollViewPageMessage::Add => {
                let idx = self.radios.len() + 1;
                init! {
                    radio: RadioButton = (&self.scroll) => {
                        text: format!("Radio {idx}"),
                        checked: false,
                    },
                }
                self.radios.push(radio);
                Ok(true)
            }
            ScrollViewPageMessage::Del => {
                if !self.radios.is_empty() {
                    self.radios.pop();
                }
                Ok(true)
            }
            ScrollViewPageMessage::Show => {
                let selected = self.radios.iter().find_map(|r| {
                    if r.is_checked().unwrap_or_default() {
                        Some(r.text().unwrap_or_default())
                    } else {
                        None
                    }
                });
                sender.output(ScrollViewPageEvent::ShowMessage(
                    MessageBox::new()
                        .title("Selected Radio")
                        .message(selected.unwrap_or("No selection".to_string()))
                        .buttons(MessageBoxButton::Ok),
                ));
                Ok(false)
            }
            ScrollViewPageMessage::Select(idx) => {
                self.selected = Some(idx);
                Ok(false)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.size()?;

        let mut radios_panel = StackPanel::new(Orient::Vertical);
        for radio in self.radios.iter_mut() {
            radios_panel
                .push(radio)
                .margin(Margin::new_all_same(4.0))
                .finish();
        }

        radios_panel.set_size(csize)?;

        let mut buttons_panel = layout! {
            StackPanel::new(Orient::Vertical),
            self.add_btn  => { margin: Margin::new_all_same(4.0) },
            self.del_btn  => { margin: Margin::new_all_same(4.0) },
            self.show_btn => { margin: Margin::new_all_same(4.0) },
        };

        let mut root_panel = layout! {
            Grid::from_str("1*,auto", "1*").unwrap(),
            self.scroll   => { column: 0, row: 0 },
            buttons_panel => { column: 1, row: 0, halign: HAlign::Center, valign: VAlign::Top },
        };

        root_panel.set_size(csize)?;
        Ok(())
    }
}

impl Deref for ScrollViewPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
