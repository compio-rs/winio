use winio::prelude::*;

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new("rs.compio.winio.scrollview").run::<MainModel>(());
}

struct MainModel {
    window: Child<Window>,
    scroll: Child<ScrollView>,
    radios: Vec<Child<RadioButton>>,
    add_btn: Child<Button>,
    del_btn: Child<Button>,
    show_btn: Child<Button>,
    selected: Option<usize>,
}

#[derive(Debug)]
enum MainMessage {
    Noop,
    Redraw,
    Close,
    Add,
    Del,
    Show,
    Select(usize),
}

impl Component for MainModel {
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "ScrollView Example",
                size: Size::new(400.0, 300.0),
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

        window.show();

        Self {
            window,
            scroll,
            radios,
            add_btn,
            del_btn,
            show_btn,
            selected: None,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let radios = self.radios.iter_mut().map(|r| &mut **r).collect::<Vec<_>>();
        let mut radio_group = RadioButtonGroup::new(radios);
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.add_btn => {
                ButtonEvent::Click => MainMessage::Add,
            },
            self.del_btn => {
                ButtonEvent::Click => MainMessage::Del,
            },
            self.show_btn => {
                ButtonEvent::Click => MainMessage::Show,
            },
            self.scroll => {},
            radio_group => {
                |i| Some(MainMessage::Select(i))
            }
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        let (b1, b2) = futures_util::future::join(self.window.update(), self.scroll.update()).await;
        let b3 = match message {
            MainMessage::Noop => false,
            MainMessage::Redraw => true,
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Add => {
                let idx = self.radios.len() + 1;
                init! {
                    radio: RadioButton = (&self.scroll) => {
                        text: format!("Radio {idx}"),
                        checked: false,
                    },
                }
                self.radios.push(radio);
                true
            }
            MainMessage::Del => {
                if !self.radios.is_empty() {
                    self.radios.pop();
                }
                true
            }
            MainMessage::Show => {
                let selected = self
                    .radios
                    .iter()
                    .find_map(|r| if r.is_checked() { Some(r.text()) } else { None });
                MessageBox::new()
                    .title("Selected Radio")
                    .message(selected.unwrap_or("No selection".to_string()))
                    .buttons(MessageBoxButton::Ok)
                    .show(&self.window)
                    .await;
                false
            }
            MainMessage::Select(idx) => {
                self.selected = Some(idx);
                false
            }
        };
        b1 | b2 | b3
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.client_size();

        let mut radios_panel = StackPanel::new(Orient::Vertical);
        for radio in self.radios.iter_mut() {
            radios_panel.push(radio).finish();
        }

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

        root_panel.set_size(csize);

        let scroll_size = self.scroll.size();
        radios_panel.set_size(scroll_size);
        self.scroll.set_size(scroll_size);
    }
}
