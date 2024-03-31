# Winio

Winio is a single-threaded asynchronous GUI runtime.
It is based on [`compio`](https://github.com/compio-rs/compio), and the GUI part is powered by Win32, GTK and Cocoa.
All IO requests could be issued in the same thread as GUI, without blocking the user interface!

## Quick start
```rust
winio::block_on(async {
    let window = Window::new().unwrap();
    window.set_text("Basic example").unwrap();
    window.set_size(Size::new(800.0, 600.0)).unwrap();

    // Wait for the close request.
    window.wait_close().await;
})
```
The asynchronous style of `winio` enables you writing simple code to handle the requests:
```rust
loop {
    // Wait for the close request.
    window.wait_close().await;
    // If the close button clicked, show a message box.
    if MessageBox::new()
        .title("Basic example")
        .message("Close window?")
        .style(MessageBoxStyle::Info)
        .buttons(MessageBoxButton::Yes | MessageBoxButton::No)
        .show(Some(&window))
        .await
        .unwrap()
        == MessageBoxResponse::Yes
    {
        // The user clicked "Yes", break the loop.
        break;
    }
}
// When `window` is dropped, it is closed immediately.
```
