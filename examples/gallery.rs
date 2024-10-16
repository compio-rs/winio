use std::path::{Path, PathBuf};

use compio::runtime::spawn_blocking;
use image::{ImageReader, RgbaImage};
use winio::{
    App, BrushPen, Button, ButtonEvent, Canvas, Child, Color, ColorTheme, Component,
    ComponentSender, Edit, FileBox, Layoutable, Orient, Point, Rect, Size, SolidColorBrush,
    StackPanel, Window, WindowEvent,
};

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new().run::<MainModel>(dirs::picture_dir(), &())
}

struct MainModel {
    window: Child<Window>,
    canvas: Child<Canvas>,
    button: Child<Button>,
    entry: Child<Edit>,
    images: Vec<RgbaImage>,
    is_dark: bool,
}

#[derive(Debug)]
enum MainMessage {
    Close,
    Redraw,
    ChooseFolder,
    OpenFolder(PathBuf),
    Clear,
    Append(RgbaImage),
}

impl Component for MainModel {
    type Event = ();
    type Init = Option<PathBuf>;
    type Message = MainMessage;
    type Root = ();

    fn init(path: Self::Init, _root: &Self::Root, sender: &winio::ComponentSender<Self>) -> Self {
        let mut window = Child::<Window>::init((), &());
        window.set_text("Gallery example");
        window.set_size(Size::new(800.0, 600.0));

        let is_dark = ColorTheme::current() == ColorTheme::Dark;

        let canvas = Child::<Canvas>::init((), &window);
        let mut button = Child::<Button>::init((), &window);
        button.set_text("...");

        let mut entry = Child::<Edit>::init((), &window);
        entry.set_text(
            path.as_ref()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
        );

        if let Some(path) = path {
            let sender = sender.clone();
            spawn_blocking(move || fetch(path, sender)).detach();
        }

        Self {
            window,
            canvas,
            button,
            entry,
            images: vec![],
            is_dark,
        }
    }

    async fn start(&mut self, sender: &winio::ComponentSender<Self>) {
        let fut_window = self.window.start(sender, |e| match e {
            WindowEvent::Close => Some(MainMessage::Close),
            WindowEvent::Resize => Some(MainMessage::Redraw),
            _ => None,
        });
        let fut_button = self.button.start(sender, |e| match e {
            ButtonEvent::Click => Some(MainMessage::ChooseFolder),
            _ => None,
        });
        futures_util::future::join(fut_window, fut_button).await;
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &winio::ComponentSender<Self>,
    ) -> bool {
        futures_util::future::join3(
            self.window.update(),
            self.canvas.update(),
            self.button.update(),
        )
        .await;
        match message {
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Redraw => true,
            MainMessage::ChooseFolder => {
                if let Some(p) = FileBox::new()
                    .title("Open folder")
                    .open_folder(Some(&*self.window))
                    .await
                {
                    sender.post(MainMessage::OpenFolder(p));
                }
                false
            }
            MainMessage::OpenFolder(p) => {
                self.entry.set_text(p.to_str().unwrap_or_default());
                let sender = sender.clone();
                spawn_blocking(move || fetch(p, sender)).detach();
                true
            }
            MainMessage::Clear => {
                self.images.clear();
                true
            }
            MainMessage::Append(image) => {
                self.images.push(image);
                true
            }
        }
    }

    fn render(&mut self, _sender: &winio::ComponentSender<Self>) {
        let csize = self.window.client_size();

        {
            let mut root_panel = StackPanel::new(Orient::Vertical);
            let mut header_panel = StackPanel::new(Orient::Horizontal);
            header_panel.push(&mut self.entry).grow(true).finish();
            header_panel.push(&mut self.button).finish();
            root_panel.push(&mut header_panel).finish();
            root_panel.push(&mut self.canvas).grow(true).finish();
            root_panel.set_size(csize);
        }

        let size = self.canvas.size();
        let mut ctx = self.canvas.context();
        let brush = SolidColorBrush::new(if self.is_dark {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(0, 0, 0, 255)
        });
        let pen = BrushPen::new(&brush, 1.0);

        const MAX_COLUMN: usize = 6;
        let length = size.width / (MAX_COLUMN as f64);
        let content_length = length - 10.0;
        for (i, image) in self.images.iter().enumerate() {
            let c = i % MAX_COLUMN;
            let r = i / MAX_COLUMN;
            let x = c as f64 * length + 5.0;
            let y = r as f64 * length + 5.0;
            let rect = Rect::new(Point::new(x, y), Size::new(content_length, content_length));
            let rate = content_length / image.width().max(image.height()) as f64;
            let real_width = image.width() as f64 * rate;
            let real_height = image.height() as f64 * rate;
            let real_x = (content_length - real_width) / 2.0;
            let real_y = (content_length - real_height) / 2.0;
            let real_rect = Rect::new(
                Point::new(x + real_x, y + real_y),
                Size::new(real_width, real_height),
            );
            ctx.draw_image(image, real_rect, None);
            ctx.draw_rect(&pen, rect);
        }
    }
}

fn fetch(path: impl AsRef<Path>, sender: ComponentSender<MainModel>) {
    sender.post(MainMessage::Clear);
    let mut counter = 0;
    for p in path.as_ref().read_dir().unwrap() {
        if counter >= 10 {
            break;
        }
        let p = p.unwrap();
        if let Ok(reader) = ImageReader::open(p.path()) {
            if let Ok(image) = reader.decode() {
                sender.post(MainMessage::Append(image.to_rgba8()));
                counter += 1;
            }
        }
    }
}
