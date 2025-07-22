use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use compio::runtime::spawn_blocking;
use image::{DynamicImage, ImageReader};
use itertools::Itertools;
use winio::prelude::*;

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new("rs.compio.winio.gallery").run::<MainModel>(dirs::picture_dir())
}

struct MainModel {
    window: Child<Window>,
    canvas: Child<Canvas>,
    scrollbar: Child<ScrollBar>,
    button: Child<Button>,
    entry: Child<Edit>,
    list: Child<ObservableVec<String>>,
    listbox: Child<ListBox>,
    images: Vec<DynamicImage>,
    sel_images: BTreeMap<usize, Option<DrawingImage>>,
}

const MAX_COLUMN: usize = 3;

impl MainModel {
    fn update_scrollbar(&mut self) {
        let pos = self.scrollbar.pos();

        let size = self.canvas.size();
        let occupy_width = size.width / (MAX_COLUMN as f64);
        let content_width = occupy_width - 10.0;
        let content_height: f64 = self
            .sel_images
            .keys()
            .chunks(MAX_COLUMN)
            .into_iter()
            .map(|images| {
                (images
                    .map(|i| {
                        let image = &self.images[*i];
                        let image_size = Size::new(image.width() as _, image.height() as _);
                        (image_size.height
                            * (content_width / image_size.width.max(image_size.height)))
                            as usize
                    })
                    .max()
                    .unwrap_or_default() as f64)
                    .min(content_width)
                    + 10.0
            })
            .sum();
        self.scrollbar.set_maximum(content_height as _);
        self.scrollbar.set_page(size.height as _);
        self.scrollbar.set_pos(pos);
    }
}

#[derive(Debug)]
enum MainMessage {
    Noop,
    Close,
    Redraw,
    ChooseFolder,
    OpenFolder(PathBuf),
    Clear,
    Append(PathBuf, DynamicImage),
    List(ObservableVecEvent<String>),
    Select,
    Wheel(Vector),
}

impl Component for MainModel {
    type Event = ();
    type Init<'a> = Option<PathBuf>;
    type Message = MainMessage;

    fn init(path: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "Gallery example",
                size: Size::new(800.0, 600.0),
            },
            canvas: Canvas = (&window),
            scrollbar: ScrollBar = (&window) => {
                orient: Orient::Vertical,
                minimum: 0,
            },
            button: Button = (&window) => {
                text: "...",
            },
            entry: Edit = (&window) => {
                text: path.as_ref()
                          .map(|p| p.to_string_lossy().into_owned())
                          .unwrap_or_default(),
            },
            list: ObservableVec<String> = (()),
            listbox: ListBox = (&window),
        }

        if let Some(path) = path {
            let sender = sender.clone();
            spawn_blocking(move || fetch(path, sender)).detach();
        }

        window.show();

        Self {
            window,
            canvas,
            scrollbar,
            button,
            entry,
            list,
            listbox,
            images: vec![],
            sel_images: BTreeMap::new(),
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize => MainMessage::Redraw,
            },
            self.canvas => {
                CanvasEvent::MouseWheel(w) => MainMessage::Wheel(w),
            },
            self.button => {
                ButtonEvent::Click => MainMessage::ChooseFolder,
            },
            self.list => {
                e => MainMessage::List(e),
            },
            self.listbox => {
                ListBoxEvent::Select => MainMessage::Select,
            },
            self.scrollbar => {
                ScrollBarEvent::Change => MainMessage::Redraw,
            }
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        futures_util::future::join3(
            self.window.update(),
            self.canvas.update(),
            self.button.update(),
        )
        .await;
        self.update_scrollbar();
        match message {
            MainMessage::Noop => false,
            MainMessage::Close => {
                sender.output(());
                false
            }
            MainMessage::Redraw => true,
            MainMessage::ChooseFolder => {
                if let Some(p) = FileBox::new()
                    .title("Open folder")
                    .open_folder(&self.window)
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
                self.list.clear();
                self.images.clear();
                self.sel_images.clear();
                true
            }
            MainMessage::Append(path, image) => {
                if let Some(filename) = path.file_name() {
                    self.list.push(filename.to_string_lossy().into_owned());
                    self.images.push(image);
                    true
                } else {
                    false
                }
            }
            MainMessage::List(e) => {
                self.listbox
                    .emit(ListBoxMessage::from_observable_vec_event(e))
                    .await
            }
            MainMessage::Select => {
                for i in 0..self.list.len() {
                    if self.listbox.is_selected(i) {
                        self.sel_images.entry(i).or_insert(None);
                    } else {
                        self.sel_images.remove(&i);
                    }
                }
                true
            }
            MainMessage::Wheel(w) => {
                let delta = w.y;
                let pos = self.scrollbar.pos();
                self.scrollbar
                    .set_pos((pos as f64 - delta).max(0.0) as usize);
                true
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.client_size();

        {
            let mut header_panel = layout! {
                StackPanel::new(Orient::Horizontal),
                self.entry => { grow: true },
                self.button
            };
            let mut content_panel = layout! {
                StackPanel::new(Orient::Horizontal),
                self.listbox,
                self.canvas => { grow: true },
                self.scrollbar,
            };
            let mut root_panel = layout! {
                StackPanel::new(Orient::Vertical),
                header_panel,
                content_panel => { grow: true },
            };
            root_panel.set_size(csize);
        }

        let pos = self.scrollbar.pos();

        let size = self.canvas.size();
        let mut ctx = self.canvas.context();
        let is_dark = ColorTheme::current() == ColorTheme::Dark;
        let brush = SolidColorBrush::new(if is_dark {
            Color::new(255, 255, 255, 255)
        } else {
            Color::new(0, 0, 0, 255)
        });
        let pen = BrushPen::new(&brush, 1.0);

        let occupy_width = size.width / (MAX_COLUMN as f64);
        let content_width = occupy_width - 10.0;
        for (i, image) in self.sel_images.iter_mut() {
            if image.is_none() {
                let cache = ctx.create_image(self.images[*i].clone());
                *image = Some(cache);
            }
        }
        let content_heights = self
            .sel_images
            .values()
            .chunks(MAX_COLUMN)
            .into_iter()
            .map(|images| {
                (images
                    .map(|image| {
                        let image_size = image.as_ref().unwrap().size();
                        (image_size.height
                            * (content_width / image_size.width.max(image_size.height)))
                            as usize
                    })
                    .max()
                    .unwrap_or_default() as f64)
                    .min(content_width)
            })
            .collect::<Vec<_>>();
        for (i, image) in self.sel_images.values().enumerate() {
            let image = image.as_ref().unwrap();
            let image_size = image.size();
            let c = i % MAX_COLUMN;
            let r = i / MAX_COLUMN;
            let x = c as f64 * occupy_width + 5.0;
            let y = content_heights[..r].iter().map(|h| h + 10.0).sum::<f64>() + 5.0 - pos as f64;
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
        let p = p.unwrap().path();
        if let Ok(reader) = ImageReader::open(&p) {
            if let Ok(image) = reader.decode() {
                sender.post(MainMessage::Append(p, image));
            }
        }
    }
}
