#[cfg(feature = "compio-compat")]
use std::path::PathBuf;
use std::{future::Future, pin::Pin};

pub use anyhow::{Error, Result};
use winio::prelude::*;

#[cfg(target_os = "android")]
mod android;

mod subviews;
use subviews::*;

pub struct MainModel {
    window: Child<Window>,
    tabview: Child<TabView>,
    misc: Child<MiscPage>,
    fs: Child<FsPage>,
    net: Child<NetPage>,
    gallery: Child<GalleryPage>,
    scroll: Child<ScrollViewPage>,
    plotters: Child<PlottersPage>,
    wgpu: Child<WgpuPage>,
    media: Child<MediaPage>,
    webview: Child<WebViewPage>,
    markdown: Child<MarkdownPage>,
}

#[derive(Debug)]
pub enum MainMessage {
    Noop,
    Close,
    Redraw,
    #[cfg(feature = "compio-compat")]
    ChooseFile,
    #[cfg(feature = "compio-compat")]
    ChooseSaveFile,
    #[cfg(feature = "compio-compat")]
    OpenFile(PathBuf),
    #[cfg(feature = "compio-compat")]
    SaveFile(PathBuf),
    #[cfg(feature = "compio-compat")]
    ChooseFolder,
    #[cfg(feature = "compio-compat")]
    OpenFolder(PathBuf),
    ShowMessage(MessageBox),
    #[cfg(all(feature = "media", feature = "compio-compat"))]
    ChooseMedia,
    #[cfg(all(feature = "media", feature = "compio-compat"))]
    OpenMedia(PathBuf),
    #[cfg(all(feature = "webview", feature = "compio-compat"))]
    ChooseMarkdown,
    #[cfg(all(feature = "webview", feature = "compio-compat"))]
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

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: Window = (()) => {
                text: "Widgets example",
                size: Size::new(800.0, 600.0),
                loc: {
                    let monitors = Monitor::all()?;
                    if let Some(monitor) = monitors.first() {
                        let region = monitor.client_scaled();
                        region.origin + region.size / 2.0 - window.size()? / 2.0
                    } else {
                        Point::zero()
                    }
                },
            },
            tabview: TabView = (&window),
            misc: MiscPage = (()),
            #[cfg(feature = "compio-compat")]
            fs: FsPage = (()),
            #[cfg(not(feature = "compio-compat"))]
            fs: DummyPage = (("File IO", "compio-compat")),
            #[cfg(feature = "compio-compat")]
            net: NetPage = (()),
            #[cfg(not(feature = "compio-compat"))]
            net: DummyPage = (("Networking", "compio-compat")),
            #[cfg(feature = "compio-compat")]
            gallery: GalleryPage = (()),
            #[cfg(not(feature = "compio-compat"))]
            gallery: DummyPage = (("Images", "compio-compat")),
            scroll: ScrollViewPage = (()),
            #[cfg(feature = "plotters")]
            plotters: PlottersPage = (()),
            #[cfg(not(feature = "plotters"))]
            plotters: DummyPage = (("Plotters", "plotters")),
            #[cfg(feature = "wgpu")]
            wgpu: WgpuPage = (()),
            #[cfg(not(feature = "wgpu"))]
            wgpu: DummyPage = (("WGPU", "wgpu")),
            #[cfg(all(feature = "media", feature = "compio-compat"))]
            media: MediaPage = (()),
            #[cfg(not(all(feature = "media", feature = "compio-compat")))]
            media: DummyPage = (("Media", "media,compio-compat")),
            #[cfg(feature = "webview")]
            webview: WebViewPage = (()),
            #[cfg(not(feature = "webview"))]
            webview: DummyPage = (("WebView", "webview")),
            #[cfg(all(feature = "webview", feature = "compio-compat"))]
            markdown: MarkdownPage = (()),
            #[cfg(not(all(feature = "webview", feature = "compio-compat")))]
            markdown: DummyPage = (("Markdown", "webview,compio-compat")),
        }

        tabview.push(&misc)?;
        tabview.push(&fs)?;
        tabview.push(&net)?;
        tabview.push(&gallery)?;
        tabview.push(&scroll)?;
        tabview.push(&plotters)?;
        tabview.push(&wgpu)?;
        tabview.push(&media)?;
        tabview.push(&webview)?;
        tabview.push(&markdown)?;

        #[cfg(target_os = "android")]
        android::init_rustls(&window)?;

        window.show()?;

        Ok(Self {
            window,
            tabview,
            misc,
            fs,
            net,
            gallery,
            scroll,
            plotters,
            wgpu,
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
                #[cfg(feature = "compio-compat")]
                FsPageEvent::ChooseFile => MainMessage::ChooseFile,
                #[cfg(feature = "compio-compat")]
                FsPageEvent::SaveFile => MainMessage::ChooseSaveFile,
            },
            self.net => {},
            self.gallery => {
                #[cfg(feature = "compio-compat")]
                GalleryPageEvent::ChooseFolder => MainMessage::ChooseFolder,
            },
            self.scroll => {
                ScrollViewPageEvent::ShowMessage(mb) => MainMessage::ShowMessage(mb),
            },
            self.wgpu => {},
            self.media => {
                #[cfg(all(feature = "media", feature = "compio-compat"))]
                MediaPageEvent::ChooseFile => MainMessage::ChooseMedia,
                #[cfg(all(feature = "media", feature = "compio-compat"))]
                MediaPageEvent::ShowMessage(mb) => MainMessage::ShowMessage(mb),
            },
            self.webview => {},
            self.markdown => {
                #[cfg(all(feature = "webview", feature = "compio-compat"))]
                MarkdownPageEvent::ChooseFile => MainMessage::ChooseMarkdown,
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
            Box::pin(self.plotters.update()),
            Box::pin(self.wgpu.update()),
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
                futures_util::future::try_join_all(subviews.into_iter()).map_ok(|_| false),
            )
        } else {
            try_join_update!(
                self.window.update(),
                self.tabview.update(),
                futures_util::future::try_join_all(subviews.into_iter()).map_ok(|_| false)
            )
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
                if cfg!(any(target_os = "ios", target_os = "android")) {
                    sender.output(());
                } else {
                    match MessageBox::new()
                        .title("Example")
                        .message("Close window?")
                        .instruction("The window is about to close.")
                        .style(MessageBoxStyle::Info)
                        .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
                        .custom_button(CustomButton::new(114, "114"))
                        .show(&self.window)?
                        .await?
                    {
                        MessageBoxResponse::Yes | MessageBoxResponse::Custom(114) => {
                            sender.output(());
                        }
                        _ => {}
                    }
                }
                Ok(false)
            }
            MainMessage::Redraw => {
                #[cfg(feature = "compio-compat")]
                {
                    self.gallery.emit(GalleryPageMessage::Redraw).await
                }
                #[cfg(not(feature = "compio-compat"))]
                {
                    Ok(true)
                }
            }
            #[cfg(feature = "compio-compat")]
            MainMessage::ChooseFile => {
                if let Some(p) = FileBox::new()
                    .title("Open file")
                    .add_filter(("All files", "*.*"))
                    .open(&self.window)?
                    .await?
                {
                    sender.post(MainMessage::OpenFile(p));
                }
                Ok(false)
            }
            #[cfg(feature = "compio-compat")]
            MainMessage::ChooseSaveFile => {
                if let Some(p) = FileBox::new()
                    .title("Save file")
                    .add_filter(("All files", "*.*"))
                    .save(&self.window)?
                    .await?
                {
                    sender.post(MainMessage::SaveFile(p));
                }
                Ok(false)
            }
            #[cfg(feature = "compio-compat")]
            MainMessage::OpenFile(p) => self.fs.emit(FsPageMessage::OpenFile(p)).await,
            #[cfg(feature = "compio-compat")]
            MainMessage::SaveFile(p) => self.fs.emit(FsPageMessage::SaveFile(p)).await,
            #[cfg(feature = "compio-compat")]
            MainMessage::ChooseFolder => {
                if let Some(p) = FileBox::new()
                    .title("Open folder")
                    .open_folder(&self.window)?
                    .await?
                {
                    sender.post(MainMessage::OpenFolder(p));
                }
                Ok(false)
            }
            #[cfg(feature = "compio-compat")]
            MainMessage::OpenFolder(p) => {
                self.gallery.emit(GalleryPageMessage::OpenFolder(p)).await
            }
            MainMessage::ShowMessage(mb) => {
                mb.show(&self.window)?.await?;
                Ok(false)
            }
            #[cfg(all(feature = "media", feature = "compio-compat"))]
            MainMessage::ChooseMedia => {
                if let Some(p) = FileBox::new()
                    .title("Open media file")
                    .add_filter(("MP4 video", "*.mp4"))
                    .add_filter(("All files", "*.*"))
                    .open(&self.window)?
                    .await?
                {
                    sender.post(MainMessage::OpenMedia(p));
                }
                Ok(false)
            }
            #[cfg(all(feature = "media", feature = "compio-compat"))]
            MainMessage::OpenMedia(p) => self.media.emit(MediaPageMessage::OpenFile(p)).await,
            #[cfg(all(feature = "webview", feature = "compio-compat"))]
            MainMessage::ChooseMarkdown => {
                if let Some(p) = FileBox::new()
                    .title("Open markdown file")
                    .add_filter(("Markdown files", "*.md"))
                    .add_filter(("All files", "*.*"))
                    .open(&self.window)?
                    .await?
                {
                    sender.post(MainMessage::OpenMarkdown(p));
                }
                Ok(false)
            }
            #[cfg(all(feature = "webview", feature = "compio-compat"))]
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
                5 => self.plotters.render()?,
                6 => self.wgpu.render()?,
                7 => self.media.render()?,
                8 => self.webview.render()?,
                9 => self.markdown.render()?,
                _ => {}
            }
        }
        Ok(())
    }
}
