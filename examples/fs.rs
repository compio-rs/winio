use compio_io::AsyncReadAtExt;
use winio::{block_on, fs::File};

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::DEBUG)
        .init();

    block_on(async {
        let file = File::open("Cargo.toml").unwrap();
        let (_, buffer) = file.read_to_end_at(vec![], 0).await.unwrap();
        println!("{}", std::str::from_utf8(&buffer).unwrap());
    })
}
