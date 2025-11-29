use std::{convert::Infallible, future::Future, path::PathBuf, pin::Pin};

use compio_log::error;
use futures_util::StreamExt;
use thiserror::Error;
use winio::prelude::*;

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
    /// Image error.
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
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

impl From<std::io::ErrorKind> for Error {
    fn from(e: std::io::ErrorKind) -> Self {
        Error::Io(e.into())
    }
}

impl From<Infallible> for Error {
    fn from(e: Infallible) -> Self {
        match e {}
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

fn main() -> Result<()> {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    let app = App::new("rs.compio.winio.widgets")?;

    app.block_on(async {
        let mut model = Root::<MainModel>::init(())?;
        let stream = model.run();
        let mut stream = std::pin::pin!(stream);
        while let Some(event) = stream.next().await {
            match event {
                RunEvent::Event(()) => return Ok(()),
                RunEvent::UpdateErr(_e) => {
                    error!("Update error: {_e:?}");
                }
                RunEvent::RenderErr(_e) => {
                    error!("Render error: {_e:?}");
                }
                _ => {
                    error!("Unexpected event: {event:?}");
                }
            }
        }
        unreachable!("Component ended unexpectedly");
    })
}

mod subviews;
use subviews::*;

struct MainModel {
    window: Child<Window>,
    tabview: Child<TabView>,
    misc: Child<MiscPage>,
    fs: Child<FsPage>,
    net: Child<NetPage>,
    gallery: Child<GalleryPage>,
    scroll: Child<ScrollViewPage>,
    #[cfg(feature = "media")]
    media: Child<MediaPage>,
    #[cfg(not(feature = "media"))]
    media: Child<DummyPage>,
    #[cfg(feature = "webview")]
    webview: Child<WebViewPage>,
    #[cfg(not(feature = "webview"))]
    webview: Child<DummyPage>,
    #[cfg(feature = "webview")]
    markdown: Child<MarkdownPage>,
    #[cfg(not(feature = "webview"))]
    markdown: Child<DummyPage>,
}

#[derive(Debug)]
enum MainMessage {
    Noop,
    Close,
    Redraw,
    ChooseFile,
    OpenFile(PathBuf),
    ChooseFolder,
    OpenFolder(PathBuf),
    ShowMessage(MessageBox),
    #[cfg(feature = "media")]
    ChooseMedia,
    #[cfg(feature = "media")]
    OpenMedia(PathBuf),
    #[cfg(feature = "webview")]
    ChooseMarkdown,
    #[cfg(feature = "webview")]
    OpenMarkdown(PathBuf),
    #[cfg(windows)]
    ChooseBackdrop(Backdrop),
    #[cfg(target_os = "macos")]
    ChooseVibrancy(Option<Vibrancy>),
}

impl Component for MainModel {
    type Error = Error;
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    fn init(_init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: Window = (()) => {
                text: "Widgets example",
                size: Size::new(800.0, 600.0),
                loc: {
                    let monitors = Monitor::all()?;
                    let region = monitors[0].client_scaled();
                    region.origin + region.size / 2.0 - window.size()? / 2.0
                },
            },
            tabview: TabView = (&window),
            misc: MiscPage = (&*tabview),
            fs: FsPage = (&*tabview),
            net: NetPage = (&*tabview),
            gallery: GalleryPage = (&*tabview),
            scroll: ScrollViewPage = (&*tabview),
            #[cfg(feature = "media")]
            media: MediaPage = (&*tabview),
            #[cfg(not(feature = "media"))]
            media: DummyPage = ((&*tabview, "Media", "media")),
            #[cfg(feature = "webview")]
            webview: WebViewPage = (&*tabview),
            #[cfg(not(feature = "webview"))]
            webview: DummyPage = ((&*tabview, "WebView", "webview")),
            #[cfg(feature = "webview")]
            markdown: MarkdownPage = (&*tabview),
            #[cfg(not(feature = "webview"))]
            markdown: DummyPage = ((&*tabview, "Markdown", "webview")),
        }

        tabview.push(&misc)?;
        tabview.push(&fs)?;
        tabview.push(&net)?;
        tabview.push(&gallery)?;
        tabview.push(&scroll)?;
        tabview.push(&media)?;
        tabview.push(&webview)?;
        tabview.push(&markdown)?;

        sender.post(MainMessage::Redraw);

        window.show()?;

        Ok(Self {
            window,
            tabview,
            misc,
            fs,
            net,
            gallery,
            scroll,
            media,
            webview,
            markdown,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MainMessage::Noop,
            self.window => {
                WindowEvent::Close => MainMessage::Close,
                WindowEvent::Resize | WindowEvent::ThemeChanged => MainMessage::Redraw,
            },
            self.tabview => {
                TabViewEvent::Select => MainMessage::Redraw,
            },
            self.misc => {
                MiscPageEvent::ShowMessage(mb) => MainMessage::ShowMessage(mb),
                #[cfg(windows)]
                MiscPageEvent::ChooseBackdrop(b) => MainMessage::ChooseBackdrop(b),
                #[cfg(target_os = "macos")]
                MiscPageEvent::ChooseVibrancy(v) => MainMessage::ChooseVibrancy(v),
            },
            self.fs => {
                FsPageEvent::ChooseFile => MainMessage::ChooseFile,
            },
            self.net => {},
            self.gallery => {
                GalleryPageEvent::ChooseFolder => MainMessage::ChooseFolder,
                GalleryPageEvent::ShowMessage(mb) => MainMessage::ShowMessage(mb),
            },
            self.scroll => {
                ScrollViewPageEvent::ShowMessage(mb) => MainMessage::ShowMessage(mb),
            },
            self.media => {
                #[cfg(feature = "media")]
                MediaPageEvent::ChooseFile => MainMessage::ChooseMedia,
                #[cfg(feature = "media")]
                MediaPageEvent::ShowMessage(mb) => MainMessage::ShowMessage(mb),
            },
            self.webview => {},
            self.markdown => {
                #[cfg(feature = "webview")]
                MarkdownPageEvent::ChooseFile => MainMessage::ChooseMarkdown,
                #[cfg(feature = "webview")]
                MarkdownPageEvent::MessageBox(mb) => MainMessage::ShowMessage(mb),
            },
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        let mut subviews: Vec<Pin<Box<dyn Future<Output = Result<bool>>>>> = vec![
            Box::pin(self.misc.update()),
            Box::pin(self.fs.update()),
            Box::pin(self.net.update()),
            Box::pin(self.gallery.update()),
            Box::pin(self.scroll.update()),
            Box::pin(self.media.update()),
            Box::pin(self.webview.update()),
            Box::pin(self.markdown.update()),
        ];

        if let Some(index) = self.tabview.selection()? {
            let visible_subview = subviews.remove(index);
            try_join_update!(
                self.window.update(),
                self.tabview.update(),
                visible_subview,
                async {
                    futures_util::future::try_join_all(subviews.into_iter()).await?;
                    Ok::<_, Error>(false)
                },
            )
        } else {
            try_join_update!(self.window.update(), self.tabview.update(), async {
                futures_util::future::try_join_all(subviews.into_iter()).await?;
                Ok::<_, Error>(false)
            })
        }
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            MainMessage::Noop => Ok(false),
            MainMessage::Close => {
                match MessageBox::new()
                    .title("Example")
                    .message("Close window?")
                    .instruction("The window is about to close.")
                    .style(MessageBoxStyle::Info)
                    .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
                    .custom_button(CustomButton::new(114, "114"))
                    .show(&self.window)
                    .await?
                {
                    MessageBoxResponse::Yes | MessageBoxResponse::Custom(114) => {
                        sender.output(());
                    }
                    _ => {}
                }
                Ok(false)
            }
            MainMessage::Redraw => self.gallery.emit(GalleryPageMessage::Redraw).await,
            MainMessage::ChooseFile => {
                if let Some(p) = FileBox::new()
                    .title("Open file")
                    .add_filter(("All files", "*.*"))
                    .open(&self.window)
                    .await?
                {
                    sender.post(MainMessage::OpenFile(p));
                }
                Ok(false)
            }
            MainMessage::OpenFile(p) => self.fs.emit(FsPageMessage::OpenFile(p)).await,
            MainMessage::ChooseFolder => {
                if let Some(p) = FileBox::new()
                    .title("Open folder")
                    .open_folder(&self.window)
                    .await?
                {
                    sender.post(MainMessage::OpenFolder(p));
                }
                Ok(false)
            }
            MainMessage::OpenFolder(p) => {
                self.gallery.emit(GalleryPageMessage::OpenFolder(p)).await
            }
            MainMessage::ShowMessage(mb) => {
                mb.show(&self.window).await?;
                Ok(false)
            }
            #[cfg(feature = "media")]
            MainMessage::ChooseMedia => {
                if let Some(p) = FileBox::new()
                    .title("Open media file")
                    .add_filter(("MP4 video", "*.mp4"))
                    .add_filter(("All files", "*.*"))
                    .open(&self.window)
                    .await?
                {
                    sender.post(MainMessage::OpenMedia(p));
                }
                Ok(false)
            }
            #[cfg(feature = "media")]
            MainMessage::OpenMedia(p) => self.media.emit(MediaPageMessage::OpenFile(p)).await,
            #[cfg(feature = "webview")]
            MainMessage::ChooseMarkdown => {
                if let Some(p) = FileBox::new()
                    .title("Open markdown file")
                    .add_filter(("Markdown files", "*.md"))
                    .add_filter(("All files", "*.*"))
                    .open(&self.window)
                    .await?
                {
                    sender.post(MainMessage::OpenMarkdown(p));
                }
                Ok(false)
            }
            #[cfg(feature = "webview")]
            MainMessage::OpenMarkdown(p) => {
                self.markdown.emit(MarkdownPageMessage::OpenFile(p)).await
            }
            #[cfg(windows)]
            MainMessage::ChooseBackdrop(backdrop) => {
                self.window.set_backdrop(backdrop)?;
                Ok(true)
            }
            #[cfg(target_os = "macos")]
            MainMessage::ChooseVibrancy(vibrancy) => {
                self.window.set_vibrancy(vibrancy)?;
                Ok(true)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.client_size()?;
        {
            let mut root_panel = layout! {
                Grid::from_str("1*", "1*").unwrap(),
                self.tabview => {
                    margin: Margin::new_all_same(4.0),
                    halign: HAlign::Stretch,
                    valign: VAlign::Stretch
                }
            };
            root_panel.set_size(csize)?;
        }
        Ok(())
    }

    fn render_children(&mut self) -> Result<()> {
        if let Some(index) = self.tabview.selection()? {
            match index {
                0 => self.misc.render()?,
                1 => self.fs.render()?,
                2 => self.net.render()?,
                3 => self.gallery.render()?,
                4 => self.scroll.render()?,
                5 => self.media.render()?,
                6 => self.webview.render()?,
                7 => self.markdown.render()?,
                _ => {}
            }
        }
        Ok(())
    }
}
