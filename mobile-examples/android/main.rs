#[cfg(feature = "hello")]
#[path = "../../winio/examples/hello.rs"]
mod hello;

#[cfg(feature = "widgets")]
#[path = "../../winio/examples/widgets.rs"]
mod widgets;


#[unsafe(no_mangle)]
fn main() {
    #[cfg(feature = "enable_log")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("winio")
            .with_max_level(log::LevelFilter::Info),
    );
    #[cfg(feature = "hello")]
    hello::main();
    #[cfg(feature = "widgets")]
    widgets::main();

}