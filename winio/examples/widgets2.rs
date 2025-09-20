use winio::prelude::*;

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new("rs.compio.winio.widgets").run::<MainModel>(());
}

struct MainModel {
    window: Child<Window>,
    tabview: Child<TabView>,
    tab1: Child<TabViewItem>,
    label1: Child<Label>,
    tab2: Child<TabViewItem>,
    button2: Child<Button>,
}

#[derive(Debug)]
enum MainMessage {
    Noop,
    Close,
    Redraw,
    Click,
}

impl Component for MainModel {
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    fn init(_init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "Widgets example",
                size: Size::new(800.0, 600.0),
            },
            tabview: TabView = (&window),
            tab1: TabViewItem = (&*tabview) => {
                text: "Tab 1",
            },
            tab2: TabViewItem = (&*tabview) => {
                text: "Tab 2",
            },
            label1: Label = (&tab1) => {
                text: "Here is tab 1",
            },
            button2: Button = (&tab2) => {
                text: "Click me"
            }
        }

        tabview.insert(0, &tab1);
        tabview.insert(1, &tab2);

        sender.post(MainMessage::Redraw);

        window.show();

        Self {
            window,
            tabview,
            tab1,
            tab2,
            label1,
            button2,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.tabview => {},
            self.button2 => {
                ButtonEvent::Click => MainMessage::Click,
            }
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        let (b1, b2, b3, b4) = futures_util::future::join4(
            self.window.update(),
            self.tabview.update(),
            self.tab1.update(),
            self.tab2.update(),
        )
        .await;
        let b5 = match message {
            MainMessage::Noop => false,
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Redraw => true,
            MainMessage::Click => {
                self.window.set_text("Clicked!");
                false
            }
        };
        b1 | b2 | b3 | b4 | b5
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.client_size();
        {
            let mut root_panel = layout! {
                Grid::from_str("1*", "1*").unwrap(),
                self.tabview => {
                    margin: Margin::new_all_same(4.0),
                    halign: HAlign::Stretch,
                    valign: VAlign::Stretch
                }
            };
            root_panel.set_size(csize);
        }
        let tsize = self.tab1.size();
        {
            let mut root_panel = layout! {
                StackPanel::new(Orient::Vertical),
                self.label1 => { margin: Margin::new_all_same(4.0), halign: HAlign::Center }
            };
            root_panel.set_size(tsize);
        }
        {
            let mut root_panel = layout! {
                Grid::from_str("1*", "1*").unwrap(),
                self.button2 => {
                    margin: Margin::new_all_same(4.0),
                    halign: HAlign::Center,
                    valign: VAlign::Center
                }
            };
            root_panel.set_size(tsize);
        }
    }
}
