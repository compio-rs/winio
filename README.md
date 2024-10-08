# Winio

Winio is a single-threaded asynchronous GUI runtime.
It is based on [`compio`](https://github.com/compio-rs/compio), and the GUI part is powered by Win32, Qt, GTK and Cocoa.
All IO requests could be issued in the same thread as GUI, without blocking the user interface!

## Quick start

Winio follows ELM-like design, inspired by [`yew`](https://yew.rs/) and [`relm4`](https://relm4.org/).
The application starts with a root `Component`:

```rust
struct MainModel {
    window: Child<Window>,
}

enum MainMessage {
    Close,
}

impl Component for MainModel {
    type Event = ();
    type Init = ();
    type Message = MainMessage;
    type Root = ();

    fn init(_counter: Self::Init, _root: &Self::Root, sender: &ComponentSender<Self>) -> Self {
        // create & initialize the window
        let mut window = Child::<Window>::init((), &());
        window.set_text("Basic example");
        window.set_size(Size::new(800.0, 600.0));
        Self { window }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) {
        // listen to events
        self.window
            .start(sender, |e| match e {
                WindowEvent::Close => Some(MainMessage::Close),
                _ => None,
            })
            .await;
    }

    async fn update(&mut self, message: Self::Message, sender: &ComponentSender<Self>) -> bool {
        // update the window
        self.window.update().await;
        // deal with custom messages
        match message {
            MainMessage::Close => {
                // the root component output stops the application
                sender.output(());
                // need not to call `render`
                false
            }
        }
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {
        // adjust layout and draw widgets here
    }
}
```
The asynchronous style of `winio` enables you writing simple code to handle the requests:
```rust
MainMessage::Close => {
    match MessageBox::new()
        .title("Basic example")
        .message("Close window?")
        .style(MessageBoxStyle::Info)
        .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
        .show(Some(&*self.window))
        .await
    {
        MessageBoxResponse::Yes => {
            // stop the application
            sender.output(());
        }
        _ => {}
    }
    false
}
```
