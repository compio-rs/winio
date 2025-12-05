pub use std::convert::Infallible as Error;
pub type Result<T, E = Error> = std::result::Result<T, E>;

fn not_impl() -> ! {
    unimplemented!("stub implementation")
}

mod runtime;
pub use runtime::*;

mod ui;
pub use ui::*;
