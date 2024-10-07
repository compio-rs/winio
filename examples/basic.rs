use std::time::Duration;

use compio::{runtime::spawn, time::interval};
use winio::{
    App, BrushPen, Canvas, CanvasEvent, CanvasMessage, Child, Color, ColorTheme, Component,
    ComponentSender, CustomButton, DrawingFontBuilder, HAlign, MessageBox, MessageBoxButton,
    MessageBoxResponse, MessageBoxStyle, MouseButton, Point, Rect, Size, SolidColorBrush, VAlign,
    Window, WindowEvent, block_on,
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::DEBUG)
        .init();

    block_on(App::new("winio.basic").run::<MainModel>(0, &()));
}

struct MainModel {
    window: Child<Window>,
    canvas: Child<Canvas>,
    counter: usize,
    is_dark: bool,
}

enum MainMessage {
    Tick,
    Close,
    QueueRedraw,
    Redraw,
    Mouse(MouseButton),
    MouseMove(Point),
}

impl Component for MainModel {
    type Event = ();
    type Init = usize;
    type Message = MainMessage;
    type Root = ();

    fn init(counter: Self::Init, _root: &Self::Root, sender: ComponentSender<Self>) -> Self {
        let mut window = Child::<Window>::init((), &());
        let canvas = Child::init((), &*window);

        window.set_text("Basic example");
        window.set_size(Size::new(800.0, 600.0));

        spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                sender.post(MainMessage::Tick).await;
            }
        })
        .detach();
        Self {
            window,
            canvas,
            counter,
            is_dark: ColorTheme::current() == ColorTheme::Dark,
        }
    }

    async fn start(&mut self, sender: ComponentSender<Self>) {
        let fut_window = self.window.start(sender.clone(), |e| match e {
            WindowEvent::Close => Some(MainMessage::Close),
            WindowEvent::Move | WindowEvent::Resize => Some(MainMessage::QueueRedraw),
            _ => None,
        });
        let fut_canvas = self.canvas.start(sender, |e| match e {
            CanvasEvent::Redraw => Some(MainMessage::Redraw),
            CanvasEvent::MouseDown(b) | CanvasEvent::MouseUp(b) => Some(MainMessage::Mouse(b)),
            CanvasEvent::MouseMove(p) => Some(MainMessage::MouseMove(p)),
            _ => None,
        });
        futures_util::future::join(fut_window, fut_canvas).await;
    }

    async fn update(&mut self, message: Self::Message, sender: ComponentSender<Self>) -> bool {
        match message {
            MainMessage::Tick => {
                self.counter += 1;
                true
            }
            MainMessage::Close => {
                match MessageBox::new()
                    .title("Basic example")
                    .message("Close window?")
                    .style(MessageBoxStyle::Info)
                    .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
                    .custom_button(CustomButton::new(114, "114"))
                    .show(Some(&*self.window))
                    .await
                    .unwrap()
                {
                    MessageBoxResponse::Yes | MessageBoxResponse::Custom(114) => {
                        sender.output(()).await;
                    }
                    _ => {}
                }
                false
            }
            MainMessage::QueueRedraw => self.canvas.update(CanvasMessage::Redraw).await,
            MainMessage::Redraw => true,
            MainMessage::Mouse(b) => {
                println!("{:?}", b);
                false
            }
            MainMessage::MouseMove(p) => {
                println!("{:?}", p);
                false
            }
        }
    }

    fn render(&mut self, _sender: ComponentSender<Self>) {
        self.window.render();
        self.canvas.render();

        let csize = self.window.client_size();
        self.canvas.set_size(csize / 2.0);
        self.canvas
            .set_loc(Point::new(csize.width / 4.0, csize.height / 4.0));

        let size = self.canvas.size();
        let brush = SolidColorBrush::new(if self.is_dark {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(0, 0, 0, 255)
        });
        let ctx = self.canvas.context();
        ctx.draw_ellipse(
            BrushPen::new(brush.clone(), 1.0),
            Rect::new(Point::new(size.width / 4.0, size.height / 4.0), size / 2.0),
        )
        .unwrap();
        ctx.draw_str(
            &brush,
            DrawingFontBuilder::new()
                .halign(HAlign::Center)
                .valign(VAlign::Center)
                .family("Arial")
                .size(12.0)
                .build(),
            Point::new(size.width / 2.0, size.height / 2.0),
            format!("Hello world!\nRunning: {}s", self.counter),
        )
        .unwrap();
    }
}
