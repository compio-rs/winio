use std::{
    cell::RefCell,
    io,
    net::SocketAddr,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
};

use axum::{http::Uri, response::IntoResponse};
use compio::{buf::buf_try, fs::File, io::AsyncReadAtExt, net::TcpListener, runtime::spawn};
use local_sync::oneshot;
use send_wrapper::SendWrapper;
use winio::prelude::*;

use crate::{Error, Result};

pub struct MarkdownPage {
    window: Child<TabViewItem>,
    webview: Child<WebView>,
    button: Child<Button>,
    label: Child<Label>,
    markdown_path: Rc<RefCell<PathBuf>>,
    addr: Option<SocketAddr>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

#[derive(Debug)]
pub enum MarkdownFetchStatus {
    Complete(String),
    Error(String),
}

#[derive(Debug)]
pub enum MarkdownPageEvent {
    ChooseFile,
    MessageBox(MessageBox),
}

#[derive(Debug)]
pub enum MarkdownPageMessage {
    Noop,
    SetAddr(SocketAddr),
    ChooseFile,
    OpenFile(PathBuf),
    Fetch(MarkdownFetchStatus),
}

impl Component for MarkdownPage {
    type Error = Error;
    type Event = MarkdownPageEvent;
    type Init<'a> = &'a TabView;
    type Message = MarkdownPageMessage;

    fn init(tabview: Self::Init<'_>, sender: &ComponentSender<Self>) -> Result<Self> {
        let path = "README.md";
        init! {
            window: TabViewItem = (tabview) => {
                text: "Markdown",
            },
            webview: WebView = (&window),
            button: Button = (&window) => {
                text: "Choose file...",
            },
            label: Label = (&window) => {
                text: path,
                halign: HAlign::Center,
            },
        }

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let markdown_path = Rc::new(RefCell::new(PathBuf::from(path)));
        {
            let markdown_path = SendWrapper::new(markdown_path.clone());
            let sender = sender.clone();
            spawn(async move {
                let listener = TcpListener::bind("127.0.0.1:0").await?;
                let serve = cyper_axum::serve(
                    listener,
                    axum::routing::get(move |req: Uri| {
                        SendWrapper::new(async move {
                            let path = req.path().trim_start_matches('/').to_string();
                            let path = markdown_path
                                .borrow()
                                .parent()
                                .unwrap_or_else(|| Path::new("."))
                                .join(path);
                            match read_file(&path).await {
                                Ok(data) => (axum::http::StatusCode::OK, data).into_response(),
                                Err(_) => (
                                    axum::http::StatusCode::NOT_FOUND,
                                    b"File not found".to_vec(),
                                )
                                    .into_response(),
                            }
                        })
                    }),
                )
                .with_graceful_shutdown(async move {
                    shutdown_rx.await.ok();
                });
                let local_addr = serve.local_addr()?;
                sender.post(MarkdownPageMessage::SetAddr(local_addr));
                serve.await
            })
            .detach();
        }

        let path = path.to_string();
        spawn(fetch(path, sender.clone())).detach();

        Ok(Self {
            window,
            webview,
            button,
            label,
            markdown_path,
            addr: None,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MarkdownPageMessage::Noop,
            self.button => {
                ButtonEvent::Click => MarkdownPageMessage::ChooseFile,
            },
        }
    }

    async fn update_children(&mut self) -> Result<bool> {
        update_children!(self.window, self.webview, self.button, self.label)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            MarkdownPageMessage::Noop => Ok(false),
            MarkdownPageMessage::SetAddr(addr) => {
                self.addr = Some(addr);
                Ok(false)
            }
            MarkdownPageMessage::ChooseFile => {
                sender.output(MarkdownPageEvent::ChooseFile);
                Ok(false)
            }
            MarkdownPageMessage::OpenFile(p) => {
                self.label.set_text(p.to_str().unwrap_or_default())?;
                *self.markdown_path.borrow_mut() = p.clone();
                spawn(fetch(p, sender.clone())).detach();
                Ok(true)
            }
            MarkdownPageMessage::Fetch(status) => {
                match status {
                    MarkdownFetchStatus::Complete(text) => {
                        let mut output = String::new();
                        pulldown_cmark::html::push_html(
                            &mut output,
                            pulldown_cmark::Parser::new_ext(&text, pulldown_cmark::Options::all())
                                .map(|event| match event {
                                    pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image {
                                        link_type,
                                        dest_url,
                                        title,
                                        id,
                                    }) => {
                                        let dest_url = if dest_url.starts_with("http://")
                                            || dest_url.starts_with("https://")
                                        {
                                            dest_url.to_string()
                                        } else if let Some(addr) = self.addr {
                                            format!("http://{}/{}", addr, dest_url)
                                        } else {
                                            dest_url.to_string()
                                        };
                                        pulldown_cmark::Event::Start(pulldown_cmark::Tag::Image {
                                            link_type,
                                            dest_url: dest_url.into(),
                                            title,
                                            id,
                                        })
                                    }
                                    _ => event,
                                }),
                        );
                        let html = format!(
                            r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Markdown Preview</title>
</head>
<body>
    <article>
        {output}
    </article>
</body>
</html>
"#
                        );
                        self.webview.set_html(html)?;
                    }
                    MarkdownFetchStatus::Error(err) => {
                        sender.output(MarkdownPageEvent::MessageBox(
                            MessageBox::new()
                                .title("Error")
                                .message("Failed to load markdown file.")
                                .instruction(&err)
                                .style(MessageBoxStyle::Error)
                                .buttons(MessageBoxButton::Ok),
                        ));
                    }
                }
                Ok(true)
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let csize = self.window.size()?;

        {
            let mut panel = layout! {
                StackPanel::new(Orient::Vertical),
                self.label, self.button,
                self.webview => { grow: true }
            };
            panel.set_size(csize)?;
        }
        Ok(())
    }
}

impl Deref for MarkdownPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl Drop for MarkdownPage {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

async fn read_file(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let file = File::open(path).await?;
    let (_, buffer) = buf_try!(@try file.read_to_end_at(vec![], 0).await);
    Ok(buffer)
}

async fn read_file_content(path: impl AsRef<Path>) -> io::Result<String> {
    let bytes = read_file(path).await?;
    String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

async fn fetch(path: impl AsRef<Path>, sender: ComponentSender<MarkdownPage>) {
    let status = match read_file_content(path).await {
        Ok(text) => MarkdownFetchStatus::Complete(text),
        Err(e) => MarkdownFetchStatus::Error(format!("{e:?}")),
    };
    sender.post(MarkdownPageMessage::Fetch(status));
}
