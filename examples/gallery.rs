use std::path::{Path, PathBuf};

use compio::runtime::spawn_blocking;
use image::{DynamicImage, ImageReader};
use winio::{
    App, BrushPen, Button, ButtonEvent, Canvas, Child, Color, ColorTheme, Component,
    ComponentSender, DrawingImage, Edit, FileBox, Layoutable, Orient, Point, Rect, Size,
    SolidColorBrush, StackPanel, Window, WindowEvent,
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
    images: Vec<Option<DynamicImage>>,
    images_cache: Vec<DrawingImage>,
    is_dark: bool,
}

#[derive(Debug)]
enum MainMessage {
    Close,
    Redraw,
    ChooseFolder,
    OpenFolder(PathBuf),
    Clear,
    Append(DynamicImage),
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
            images_cache: vec![],
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
                self.images_cache.clear();
                true
            }
            MainMessage::Append(image) => {
                self.images.push(Some(image));
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
        let occupy_width = size.width / (MAX_COLUMN as f64);
        let content_width = occupy_width - 10.0;
        for (i, image) in self.images.iter_mut().enumerate() {
            if let Some(image) = image.take() {
                let cache = ctx.create_image(image);
                self.images_cache.insert(i, cache);
            }
        }
        let content_heights = self
            .images_cache
            .chunks(MAX_COLUMN)
            .map(|images| {
                (images
                    .iter()
                    .map(|image| {
                        let image_size = image.size();
                        (image_size.height
                            * (content_width / image_size.width.max(image_size.height)))
                            as usize
                    })
                    .max()
                    .unwrap_or_default() as f64)
                    .min(content_width)
            })
            .collect::<Vec<_>>();
        for (i, image) in self.images_cache.iter().enumerate() {
            let image_size = image.size();
            let c = i % MAX_COLUMN;
            let r = i / MAX_COLUMN;
            let x = c as f64 * occupy_width + 5.0;
            let y = content_heights[..r].iter().map(|h| h + 10.0).sum::<f64>() + 5.0;
            let content_height = content_heights[r];
            let rect = Rect::new(Point::new(x, y), Size::new(content_width, content_height));
            let rate = (content_width / image_size.width).min(content_height / image_size.height);
            let real_width = image_size.width * rate;
            let real_height = image_size.height * rate;
            let real_x = (content_width - real_width) / 2.0;
            let real_y = (content_height - real_height) / 2.0;
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
    for p in path.as_ref().read_dir().unwrap() {
        let p = p.unwrap();
        if let Ok(reader) = ImageReader::open(p.path()) {
            if let Ok(image) = reader.decode() {
                sender.post(MainMessage::Append(image));
            }
        }
    }
}
