use anyhow::Result;
use winio::prelude::*;
use winio_layer_shell::{Anchors, Layer, LayerShellExt};

struct Bar {
    window: Child<Window>,
    info: Child<Label>,
    btn_exit: Child<Button>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Close,
    Refresh,
}

impl Component for Bar {
    type Error = anyhow::Error;
    type Event = ();
    type Init<'a> = ();
    type Message = Message;

    async fn init(_: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: Window = (()) => {
                text: "winio-layer-bar",
                size: Size::new(1.0, 40.0),
                layer: Layer::Top,
                anchors: Anchors::TOP | Anchors::LEFT | Anchors::RIGHT,
                exclusive_zone: Some(40.0),
                namespace: "winio-layer-shell-example",
            },
            info: Label = (&window) => {
                text: format!("Layer: {:?} | zone: {:?} | ns: {}", window.layer()?, window.exclusive_zone()?, window.namespace()?),
                halign: HAlign::Left,
            },
            btn_exit: Button = (&window) => {
                text: "Exit",
            },
        }

        window.show()?;

        Ok(Self {
            window,
            info,
            btn_exit,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        start! {
            sender,
            self.window => {
                WindowEvent::Close => Message::Close,
                WindowEvent::Resize => Message::Refresh,
            },
            self.info => {},
            self.btn_exit => { ButtonEvent::Click => Message::Close },
        }
    }

    async fn update(&mut self, msg: Self::Message, sender: &ComponentSender<Self>) -> Result<bool> {
        match msg {
            Message::Close => {
                sender.output(());
                Ok(false)
            }
            Message::Refresh => Ok(true),
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let sz = self.window.client_size()?;
        let mut layout = layout! {
            Grid::from_str("1*, auto", "auto").unwrap(),
            self.info => { column: 0, row: 0, margin: Margin::new_all_same(4.0) },
            self.btn_exit => { column: 1, row: 0, margin: Margin::new_all_same(4.0) },
        };
        layout.set_size(sz)?;
        Ok(())
    }

    fn render_children(&mut self) -> Result<()> {
        self.info.render()?;
        self.btn_exit.render()?;
        Ok(())
    }
}

#[cfg(not(target_os = "android"))]
fn main() -> anyhow::Result<()> {
    App::builder()
        .name("winio.layer.shell.bar")
        .build()?
        .block_on(Bar::run_until_event(()))
}

#[cfg(target_os = "android")]
fn main() {}
