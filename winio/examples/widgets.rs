use std::{future::Future, path::PathBuf, pin::Pin};

use tuplex::IntoArray;
use winio::prelude::*;

fn main() {
    #[cfg(feature = "enable_log")]
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    App::new("rs.compio.winio.widgets").run::<MainModel>(());
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
    type Event = ();
    type Init<'a> = ();
    type Message = MainMessage;

    fn init(_init: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
        init! {
            window: Window = (()) => {
                text: "Widgets example",
                size: Size::new(800.0, 600.0),
                loc: {
                    let monitors = Monitor::all();
                    let region = monitors[0].client_scaled();
                    region.origin + region.size / 2.0 - window.size() / 2.0
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

        tabview.push(&misc);
        tabview.push(&fs);
        tabview.push(&net);
        tabview.push(&gallery);
        tabview.push(&scroll);
        tabview.push(&media);
        tabview.push(&webview);
        tabview.push(&markdown);

        sender.post(MainMessage::Redraw);

        window.show();

        Self {
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
        }
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

    async fn update_children(&mut self) -> bool {
        let mut subviews: Vec<Pin<Box<dyn Future<Output = bool>>>> = vec![
            Box::pin(self.misc.update()),
            Box::pin(self.fs.update()),
            Box::pin(self.net.update()),
            Box::pin(self.gallery.update()),
            Box::pin(self.scroll.update()),
            Box::pin(self.media.update()),
            Box::pin(self.webview.update()),
            Box::pin(self.markdown.update()),
        ];
        if let Some(index) = self.tabview.selection() {
            let visible_subview = subviews.remove(index);
            futures_util::join!(
                self.window.update(),
                self.tabview.update(),
                visible_subview,
                async {
                    futures_util::future::join_all(subviews.into_iter()).await;
                    false
                },
            )
            .into_array()
            .into_iter()
            .any(|b| b)
        } else {
            futures_util::join!(self.window.update(), self.tabview.update(), async {
                futures_util::future::join_all(subviews.into_iter()).await;
                false
            })
            .into_array()
            .into_iter()
            .any(|b| b)
        }
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        match message {
            MainMessage::Noop => false,
            MainMessage::Close => {
                match MessageBox::new()
                    .title("Example")
                    .message("Close window?")
                    .instruction("The window is about to close.")
                    .style(MessageBoxStyle::Info)
                    .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
                    .custom_button(CustomButton::new(114, "114"))
                    .show(&self.window)
                    .await
                {
                    MessageBoxResponse::Yes | MessageBoxResponse::Custom(114) => {
                        sender.output(());
                    }
                    _ => {}
                }
                false
            }
            MainMessage::Redraw => {
                self.gallery.emit(GalleryPageMessage::Redraw).await;
                true
            }
            MainMessage::ChooseFile => {
                if let Some(p) = FileBox::new()
                    .title("Open file")
                    .add_filter(("All files", "*.*"))
                    .open(&self.window)
                    .await
                {
                    sender.post(MainMessage::OpenFile(p));
                }
                false
            }
            MainMessage::OpenFile(p) => self.fs.emit(FsPageMessage::OpenFile(p)).await,
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
                self.gallery.emit(GalleryPageMessage::OpenFolder(p)).await
            }
            MainMessage::ShowMessage(mb) => {
                mb.show(&self.window).await;
                false
            }
            #[cfg(feature = "media")]
            MainMessage::ChooseMedia => {
                if let Some(p) = FileBox::new()
                    .title("Open media file")
                    .add_filter(("MP4 video", "*.mp4"))
                    .add_filter(("All files", "*.*"))
                    .open(&self.window)
                    .await
                {
                    sender.post(MainMessage::OpenMedia(p));
                }
                false
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
                    .await
                {
                    sender.post(MainMessage::OpenMarkdown(p));
                }
                false
            }
            #[cfg(feature = "webview")]
            MainMessage::OpenMarkdown(p) => {
                self.markdown.emit(MarkdownPageMessage::OpenFile(p)).await
            }
            #[cfg(windows)]
            MainMessage::ChooseBackdrop(backdrop) => {
                self.window.set_backdrop(backdrop);
                true
            }
            #[cfg(target_os = "macos")]
            MainMessage::ChooseVibrancy(vibrancy) => {
                self.window.set_vibrancy(vibrancy);
                true
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.client_size();
        {
            let mut root_panel = layout! {
                Grid::from_str("1*", "1*").unwrap(),
                self.tabview => {
                    margin: Margin::new_all_same(4.0),
                    halign: HAlign::Stretch,
                    valign: VAlign::Stretch
                }
            };
            root_panel.set_size(csize);
        }
    }

    fn render_children(&mut self) {
        if let Some(index) = self.tabview.selection() {
            match index {
                0 => self.misc.render(),
                1 => self.fs.render(),
                2 => self.net.render(),
                3 => self.gallery.render(),
                4 => self.scroll.render(),
                5 => self.media.render(),
                6 => self.webview.render(),
                7 => self.markdown.render(),
                _ => {}
            }
        }
    }
}
