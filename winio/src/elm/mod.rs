use std::future::Future;

use winio_elm::Component;
use winio_primitive::{Point, Size};

use crate::runtime::Runtime;

/// Root application, manages the async runtime.
pub struct App {
    runtime: Runtime,
    name: Option<String>,
}

impl App {
    /// Create [`App`] with application name.
    pub fn new(name: impl AsRef<str>) -> Self {
        #[allow(unused_mut)]
        let mut runtime = Runtime::new();
        let name = name.as_ref().to_string();
        #[cfg(not(any(windows, target_vendor = "apple")))]
        runtime.set_app_id(&name);
        Self {
            runtime,
            name: Some(name),
        }
    }

    /// The application name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Block on the future till it completes.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    /// Create and manage the component, till it posts an event. The application
    /// returns the first event from the component.
    pub fn run<'a, T: Component>(&mut self, init: impl Into<T::Init<'a>>) -> T::Event {
        self.block_on(winio_elm::run::<T>(init))
    }
}

fn approx_eq_point(p1: Point, p2: Point) -> bool {
    approx_eq(p1.x, p2.x) && approx_eq(p1.y, p2.y)
}

fn approx_eq_size(s1: Size, s2: Size) -> bool {
    approx_eq(s1.width, s2.width) && approx_eq(s1.height, s2.height)
}

fn approx_eq(f1: f64, f2: f64) -> bool {
    (f1 - f2).abs() < 1.0
}

mod collection;
pub use collection::*;

mod window;
pub use window::*;

mod button;
pub use button::*;

mod edit;
pub use edit::*;

mod text_box;
pub use text_box::*;

mod label;
pub use label::*;

mod canvas;
pub use canvas::*;

mod progress;
pub use progress::*;

mod combo_box;
pub use combo_box::*;

mod list_box;
pub use list_box::*;

mod check_box;
pub use check_box::*;

mod radio_button;
pub use radio_button::*;
