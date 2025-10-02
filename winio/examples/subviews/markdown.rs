use std::{
    io,
    ops::Deref,
    path::{Path, PathBuf},
};

use compio::{buf::buf_try, fs::File, io::AsyncReadAtExt, runtime::spawn};
use tuplex::IntoArray;
use winio::prelude::*;

pub struct MarkdownPage {
    window: Child<TabViewItem>,
    webview: Child<WebView>,
    button: Child<Button>,
    label: Child<Label>,
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
    ChooseFile,
    OpenFile(PathBuf),
    Fetch(MarkdownFetchStatus),
}

impl Component for MarkdownPage {
    type Event = MarkdownPageEvent;
    type Init<'a> = &'a TabView;
    type Message = MarkdownPageMessage;

    fn init(tabview: Self::Init<'_>, sender: &ComponentSender<Self>) -> Self {
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

        let path = path.to_string();
        spawn(fetch(path, sender.clone())).detach();

        Self {
            window,
            webview,
            button,
            label,
        }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender, default: MarkdownPageMessage::Noop,
            self.button => {
                ButtonEvent::Click => MarkdownPageMessage::ChooseFile,
            },
        }
    }

    async fn update_children(&mut self) -> bool {
        futures_util::future::join4(
            self.window.update(),
            self.webview.update(),
            self.button.update(),
            self.label.update(),
        )
        .await
        .into_array()
        .into_iter()
        .any(|b| b)
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        match message {
            MarkdownPageMessage::Noop => false,
            MarkdownPageMessage::ChooseFile => {
                sender.output(MarkdownPageEvent::ChooseFile);
                false
            }
            MarkdownPageMessage::OpenFile(p) => {
                self.label.set_text(p.to_str().unwrap_or_default());
                spawn(fetch(p, sender.clone())).detach();
                true
            }
            MarkdownPageMessage::Fetch(status) => {
                match status {
                    MarkdownFetchStatus::Complete(text) => {
                        let mut output = String::new();
                        pulldown_cmark::html::push_html(
                            &mut output,
                            pulldown_cmark::Parser::new_ext(&text, pulldown_cmark::Options::all()),
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
                        self.webview.set_html(html);
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
                true
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        let csize = self.window.size();

        {
            let mut panel = layout! {
                StackPanel::new(Orient::Vertical),
                self.label, self.button,
                self.webview => { grow: true }
            };
            panel.set_size(csize);
        }
    }
}

impl Deref for MarkdownPage {
    type Target = TabViewItem;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

async fn read_file(path: impl AsRef<Path>) -> io::Result<String> {
    let file = File::open(path).await?;
    let (_, buffer) = buf_try!(@try file.read_to_end_at(vec![], 0).await);
    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

async fn fetch(path: impl AsRef<Path>, sender: ComponentSender<MarkdownPage>) {
    let status = match read_file(path).await {
        Ok(text) => MarkdownFetchStatus::Complete(text),
        Err(e) => MarkdownFetchStatus::Error(format!("{e:?}")),
    };
    sender.post(MarkdownPageMessage::Fetch(status));
}
