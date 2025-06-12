use std::time::Duration;

use compio::{runtime::spawn, time::interval};
use compio_log::info;
use winio::{
    App, BrushPen, Canvas, CanvasEvent, Child, Color, ColorTheme, Component, ComponentSender,
    CustomButton, DrawingFontBuilder, Grid, HAlign, Layoutable, MessageBox, MessageBoxButton,
    MessageBoxResponse, MessageBoxStyle, MouseButton, Point, Rect, Size, SolidColorBrush, VAlign,
    Visible, Window, WindowEvent,
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new().run::<MainModel>(0usize);
}

struct MainModel {
    window: Child<Window>,
    canvas: Child<Canvas>,
    counter: usize,
}

#[derive(Debug)]
enum MainMessage {
    Noop,
    Tick,
    Close,
    Redraw,
    Mouse(MouseButton),
    MouseMove(Point),
}

impl Component for MainModel {
    type Event = ();
    type Init<'a> = usize;
    type Message = MainMessage;

    fn init(counter: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        let mut window = Child::<Window>::init(());
        let canvas = Child::<Canvas>::init(&window);

        window.set_text("Basic example");
        window.set_size(Size::new(800.0, 600.0));

        let sender = sender.clone();
        spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                if !sender.post(MainMessage::Tick) {
                    break;
                }
            }
        })
        .detach();

        window.show();

        Self {
            window,
            canvas,
            counter,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        let fut_window = self.window.start(
            sender,
            |e| match e {
                WindowEvent::Close => Some(MainMessage::Close),
                WindowEvent::Resize => Some(MainMessage::Redraw),
                _ => None,
            },
            || MainMessage::Noop,
        );
        let fut_canvas = self.canvas.start(
            sender,
            |e| match e {
                CanvasEvent::Redraw => Some(MainMessage::Redraw),
                CanvasEvent::MouseDown(b) | CanvasEvent::MouseUp(b) => Some(MainMessage::Mouse(b)),
                CanvasEvent::MouseMove(p) => Some(MainMessage::MouseMove(p)),
                _ => None,
            },
            || MainMessage::Noop,
        );
        futures_util::future::join(fut_window, fut_canvas).await;
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        futures_util::future::join(self.window.update(), self.canvas.update()).await;
        match message {
            MainMessage::Noop => false,
            MainMessage::Tick => {
                self.counter += 1;
                true
            }
            MainMessage::Close => {
                match MessageBox::new()
                    .title("Basic example")
                    .message("Close window?")
                    .instruction("The window is about to close.")
                    .style(MessageBoxStyle::Info)
                    .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
                    .custom_button(CustomButton::new(114, "114"))
                    .show(Some(&*self.window))
                    .await
                {
                    MessageBoxResponse::Yes | MessageBoxResponse::Custom(114) => {
                        sender.output(());
                    }
                    _ => {}
                }
                false
            }
            MainMessage::Redraw => true,
            MainMessage::Mouse(_b) => {
                info!("{:?}", _b);
                false
            }
            MainMessage::MouseMove(_p) => {
                info!("{:?}", _p);
                false
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        self.window.render();
        self.canvas.render();

        let csize = self.window.client_size();
        {
            let mut grid = Grid::from_str("1*,2*,1*", "1*,2*,1*").unwrap();
            grid.push(&mut self.canvas).column(1).row(1).finish();
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
