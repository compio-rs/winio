use winio::prelude::*;

#[cfg(target_os = "android")]
mod android;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error from the UI backend.
    #[error("UI error: {0}")]
    Ui(#[from] winio::Error),
    /// An error from [`winio_layout`].
    #[error("Layout error: {0}")]
    Layout(#[from] TaffyError),
    /// An IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl<E: Into<Error> + std::fmt::Display> From<LayoutError<E>> for Error {
    fn from(e: LayoutError<E>) -> Self {
        match e {
            LayoutError::Taffy(te) => Error::Layout(te),
            LayoutError::Child(ce) => ce.into(),
            _ => Error::Io(std::io::Error::other(e.to_string())),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct MainModel {
    window: Child<Window>,
    text: Child<Label>,
    link: Child<LinkLabel>,
    canvas: Child<Canvas>,
}

#[derive(Debug)]
pub enum MainMessage {
    Noop,
    Close,
    Redraw,
}

impl Component for MainModel {
    type Error = Error;
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: Window = (()) => {
                text: "Hello example",
            },
            text: Label = (&window) => {
                text: "Hello, world!",
                halign: HAlign::Center,
            },
            link: LinkLabel = (&window) => {
                text: "Visit winio on GitHub",
                uri: "https://github.com/compio-rs/winio",
            },
            canvas: Canvas = (&window),
        }

        window.show()?;

        Ok(Self {
            window,
            text,
            link,
            canvas,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.link => {},
            self.canvas => {},
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(self.window, self.text, self.link, self.canvas)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            MainMessage::Noop => Ok(false),
            MainMessage::Close => {
                sender.output(());
                Ok(false)
            }
            MainMessage::Redraw => Ok(true),
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.client_size()?;
        {
            let mut grid = layout! {
                Grid::from_str("1*, auto, 1*", "1*, auto, auto, 1*, 1*").unwrap(),
                self.text => { column: 1, row: 1 },
                self.link => { column: 1, row: 2 },
                self.canvas => { column: 1, row: 3 },
            };
            grid.set_size(csize)?;
        }

        let size = self.canvas.size()?;
        let is_dark = ColorTheme::current()? == ColorTheme::Dark;
        let back_color = if is_dark {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(0, 0, 0, 255)
        };
        let brush = SolidColorBrush::new(back_color);
        let pen = BrushPen::new(&brush, 1.0);
        let mut ctx = self.canvas.context()?;
        let cx = size.width / 2.0;
        let cy = size.height / 2.0;
        let r = cx.min(cy) - 2.0;
        ctx.draw_pie(
            &pen,
            Rect::new(Point::new(cx - r, cy - r), Size::new(r * 2.0, r * 2.0)),
            std::f64::consts::PI,
            std::f64::consts::PI * 2.0,
        )?;

        let brush2 = LinearGradientBrush::new(
            [
                GradientStop::new(Color::new(0x87, 0xCE, 0xEB, 0xFF), 0.0),
                GradientStop::new(back_color, 1.0),
            ],
            RelativePoint::zero(),
            RelativePoint::new(0.0, 1.0),
        );
        let pen2 = BrushPen::new(&brush2, 1.0);
        ctx.draw_round_rect(
            &pen2,
            Rect::new(
                Point::new(cx - r - 1.0, cy - r - 1.0),
                Size::new(r * 2.0 + 2.0, r * 1.618 + 2.0),
            ),
            Size::new(r / 10.0, r / 10.0),
        )?;
        let mut path = ctx.create_path_builder(Point::new(cx + r + 1.0 - r / 10.0, cy))?;
        path.add_arc(
            Point::new(cx, cy + r * 0.618 + 1.0),
            Size::new(r + 1.0 - r / 10.0, r * 0.382 / 2.0),
            0.0,
            std::f64::consts::PI,
            true,
        )?;
        path.add_line(Point::new(cx - r - 1.0 + r / 10.0, cy))?;
        let path = path.build(false)?;
        ctx.draw_path(&pen, &path)?;
        let brush3 = RadialGradientBrush::new(
            [
                GradientStop::new(Color::new(0xF5, 0xF5, 0xF5, 0xFF), 0.0),
                GradientStop::new(
                    Color::accent().unwrap_or(Color::new(0xFF, 0xC0, 0xCB, 0xFF)),
                    1.0,
                ),
            ],
            RelativePoint::new(0.5, 0.5),
            RelativePoint::new(0.2, 0.5),
            RelativeSize::new(0.5, 0.5),
        );
        let font = DrawingFontBuilder::new()
            .family("Arial")
            .size(r / 5.0)
            .halign(HAlign::Center)
            .valign(VAlign::Bottom)
            .build();
        ctx.draw_str(&brush3, font, Point::new(cx, cy), "Hello world!")?;

        ctx.close()?;

        Ok(())
    }
}
