use std::{ops::Deref, time::Duration};

use compio::{runtime::spawn, time::interval};
use compio_log::info;
use winio::prelude::*;

pub struct BasicPage {
    window: Child<TabViewItem>,
    canvas: Child<Canvas>,
    counter: usize,
}

#[derive(Debug)]
pub enum BasicPageMessage {
    Noop,
    Tick,
    Mouse(MouseButton),
    MouseMove(Point),
}

impl Component for BasicPage {
    type Event = ();
    type Init<'a> = &'a TabView;
    type Message = BasicPageMessage;

    fn init(tabview: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        init! {
            window: TabViewItem = (tabview) => {
                text: "Basic",
            },
            canvas: Canvas = (&window),
        }

        let sender = sender.clone();
        spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                sender.post(BasicPageMessage::Tick);
            }
        })
        .detach();

        Self {
            window,
            canvas,
            counter: 0,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: BasicPageMessage::Noop,
            self.canvas => {
                CanvasEvent::MouseDown(b) | CanvasEvent::MouseUp(b) => BasicPageMessage::Mouse(b),
                CanvasEvent::MouseMove(p) => BasicPageMessage::MouseMove(p),
            },
        }
    }

    async fn update(&mut self, message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        futures_util::future::join(self.window.update(), self.canvas.update()).await;
        match message {
            BasicPageMessage::Noop => false,
            BasicPageMessage::Tick => {
                self.counter += 1;
                true
            }
            BasicPageMessage::Mouse(_b) => {
                info!("{:?}", _b);
                false
            }
            BasicPageMessage::MouseMove(_p) => {
                info!("{:?}", _p);
                false
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.size();
        {
            let mut grid = layout! {
                Grid::from_str("1*,2*,1*", "1*,2*,1*").unwrap(),
                self.canvas => { column: 1, row: 1 },
            };
            grid.set_size(csize);
        }

        let size = self.canvas.size();
        let is_dark = ColorTheme::current() == ColorTheme::Dark;
        let brush = SolidColorBrush::new(if is_dark {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(0, 0, 0, 255)
        });
        let mut ctx = self.canvas.context();
        ctx.draw_ellipse(
            BrushPen::new(brush.clone(), 1.0),
            Rect::new((size.to_vector() / 4.0).to_point(), size / 2.0),
        );
        ctx.draw_str(
            &brush,
            DrawingFontBuilder::new()
                .halign(HAlign::Center)
                .valign(VAlign::Center)
                .family("Arial")
                .size(12.0)
                .build(),
            (size.to_vector() / 2.0).to_point(),
            format!("Hello world!\nRunning: {}s", self.counter),
        );
    }
}

impl Deref for BasicPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
