use winio::prelude::*;

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new("rs.compio.winio.slider").run::<MainModel>(());
}

struct MainModel {
    window: Child<Window>,
    slider: Child<Slider>,
    volume_label: Child<Label>,
}

#[derive(Debug)]
enum MainMessage {
    Noop,
    Close,
    Redraw,
    Volume,
}

impl Component for MainModel {
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    fn init(_init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "Media example",
                size: Size::new(800.0, 600.0),
            },
            slider: Slider = (&window) => {
                minimum: 0,
                maximum: 100,
                pos: 100,
                freq: 20,
            },
            volume_label: Label = (&window),
        }
        sender.post(MainMessage::Volume);

        window.show();

        Self {
            window,
            slider,
            volume_label,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.slider => {
                SliderEvent::Change => MainMessage::Volume,
            },
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        self.window.update().await;
        match message {
            MainMessage::Noop => false,
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Redraw => true,
            MainMessage::Volume => {
                self.volume_label.set_text(self.slider.pos().to_string());
                true
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.window.render();

        let csize = self.window.client_size();
        {
            let mut bottom_bar = layout! {
                StackPanel::new(Orient::Horizontal),
                self.slider => { grow: true },
                self.volume_label => { valign: VAlign::Center },
            };
            let mut grid = layout! {
                Grid::from_str("1*", "1*,auto").unwrap(),
                bottom_bar => { column: 0, row: 1 },
            };
            grid.set_size(csize);
        }
    }
}
