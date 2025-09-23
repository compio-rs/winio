use std::{
    collections::BTreeMap,
    ops::Deref,
    path::{Path, PathBuf},
};

use compio::runtime::spawn_blocking;
use image::{DynamicImage, ImageReader};
use itertools::Itertools;
use tuplex::IntoArray;
use winio::prelude::*;

pub struct GalleryPage {
    window: Child<TabViewItem>,
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

impl GalleryPage {
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
pub enum GalleryPageEvent {
    ChooseFolder,
}

#[derive(Debug)]
pub enum GalleryPageMessage {
    Noop,
    Redraw,
    ChooseFolder,
    OpenFolder(PathBuf),
    Clear,
    Append(PathBuf, DynamicImage),
    List(ObservableVecEvent<String>),
    Select,
    Wheel(Vector),
}

impl Component for GalleryPage {
    type Event = GalleryPageEvent;
    type Init<'a> = &'a TabView;
    type Message = GalleryPageMessage;

    fn init(tabview: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        let path = dirs::picture_dir();
        init! {
            window: TabViewItem = (tabview) => {
                text: "Gallery",
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
            sender, default: GalleryPageMessage::Noop,
            self.canvas => {
                CanvasEvent::MouseWheel(w) => GalleryPageMessage::Wheel(w),
            },
            self.button => {
                ButtonEvent::Click => GalleryPageMessage::ChooseFolder,
            },
            self.list => {
                e => GalleryPageMessage::List(e),
            },
            self.listbox => {
                ListBoxEvent::Select => GalleryPageMessage::Select,
            },
            self.scrollbar => {
                ScrollBarEvent::Change => GalleryPageMessage::Redraw,
            }
        }
    }

    async fn update_children(&mut self) -> bool {
        futures_util::join!(
            self.window.update(),
            self.canvas.update(),
            self.scrollbar.update(),
            self.button.update(),
            self.entry.update(),
            self.list.update(),
            self.listbox.update(),
        )
        .into_array()
        .into_iter()
        .any(|b| b)
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        self.update_scrollbar();
        match message {
            GalleryPageMessage::Noop => false,
            GalleryPageMessage::Redraw => true,
            GalleryPageMessage::ChooseFolder => {
                sender.output(GalleryPageEvent::ChooseFolder);
                false
            }
            GalleryPageMessage::OpenFolder(p) => {
                self.entry.set_text(p.to_str().unwrap_or_default());
                let sender = sender.clone();
                spawn_blocking(move || fetch(p, sender)).detach();
                true
            }
            GalleryPageMessage::Clear => {
                self.list.clear();
                self.images.clear();
                self.sel_images.clear();
                true
            }
            GalleryPageMessage::Append(path, image) => {
                if let Some(filename) = path.file_name() {
                    self.list.push(filename.to_string_lossy().into_owned());
                    self.images.push(image);
                    true
                } else {
                    false
                }
            }
            GalleryPageMessage::List(e) => {
                self.listbox
                    .emit(ListBoxMessage::from_observable_vec_event(e))
                    .await
            }
            GalleryPageMessage::Select => {
                for i in 0..self.list.len() {
                    if self.listbox.is_selected(i) {
                        self.sel_images.entry(i).or_insert(None);
                    } else {
                        self.sel_images.remove(&i);
                    }
                }
                true
            }
            GalleryPageMessage::Wheel(w) => {
                let delta = w.y;
                let pos = self.scrollbar.pos();
                self.scrollbar
                    .set_pos((pos as f64 - delta).max(0.0) as usize);
                true
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.size();

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

impl Deref for GalleryPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

fn fetch(path: impl AsRef<Path>, sender: ComponentSender<GalleryPage>) {
    sender.post(GalleryPageMessage::Clear);
    for p in path.as_ref().read_dir().unwrap() {
        let p = p.unwrap().path();
        if let Ok(reader) = ImageReader::open(&p) {
            if let Ok(image) = reader.decode() {
                sender.post(GalleryPageMessage::Append(p, image));
            }
        }
    }
}
