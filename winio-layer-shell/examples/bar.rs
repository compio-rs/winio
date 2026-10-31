use anyhow::Result;
use winio::prelude::*;
use winio_layer_shell::{Anchors, Layer, LayerShellExt};

// Wayland layer-shell bar: full-width top panel (32px).
// Configured before show() so the compositor creates the surface.
struct Bar {
    window: Child<Window>,
    info: Child<Label>,
    btn_exit: Child<Button>,
}

impl Component for Bar {
    type Error = anyhow::Error;
    type Event = ();
    type Init<'a> = ();
    type Message = ();

    async fn init(_: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        init! {
            window: Window = (()) => {
                text: "winio-layer-bar",
                size: Size::new(1920.0, 32.0),
            },
            info: Label = (&window) => {
                text: "layer: top | zone: 32 | ns: winio-layer-shell-example",
                halign: HAlign::Left,
            },
            btn_exit: Button = (&window) => {
                text: "Exit",
                size: Size::new(80.0, 24.0),
            },
        }

        window.set_layer(Layer::Top)?;
        window.set_anchors(Anchors::TOP | Anchors::LEFT | Anchors::RIGHT)?;
        window.set_exclusive_zone(32)?;
        window.set_namespace("winio-layer-shell-example")?;
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
            self.window => { WindowEvent::Close => () },
            self.info => {},
            self.btn_exit => { ButtonEvent::Click => () },
        }
    }

    async fn update(
        &mut self,
        _msg: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        Ok(true)
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) -> Result<()> {
        let sz = self.window.client_size()?;
        self.info.set_rect(Rect::new(
            Point::new(16.0, 4.0),
            Size::new(sz.width - 120.0, 24.0),
        ))?;
        self.btn_exit.set_rect(Rect::new(
            Point::new(sz.width - 96.0, 4.0),
            Size::new(80.0, 24.0),
        ))?;
        Ok(())
    }

    fn render_children(&mut self) -> Result<()> {
        self.info.render()?;
        self.btn_exit.render()?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    App::builder()
        .name("winio-layer-shell-example")
        .build()?
        .block_on(Bar::run_until_event(()))
}
