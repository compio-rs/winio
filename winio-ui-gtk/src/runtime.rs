use std::{
    future::Future,
    task::{RawWaker, RawWakerVTable, Waker},
};

use gtk4::{
    gio::{self, prelude::ApplicationExt},
    glib::{
        MainContext,
        ffi::GMainContext,
        translate::{FromGlibPtrBorrow, FromGlibPtrFull, IntoGlibPtr},
    },
};

use crate::Result;

pub struct App {
    app: gio::Application,
    ctx: MainContext,
}

impl App {
    pub fn new() -> Result<Self> {
        let ctx = MainContext::default();
        gtk4::init()?;
        let app = gio::Application::new(None, gio::ApplicationFlags::FLAGS_NONE);
        app.set_default();

        Ok(Self { app, ctx })
    }

    pub fn set_app_id(&mut self, name: &str) -> Result<()> {
        self.app.set_application_id(Some(name));
        Ok(())
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        winio_pollable::block_on(future, glib_waker(self.ctx.clone()), || {
            self.ctx.iteration(true);
        })
    }
}

fn glib_waker(ctx: MainContext) -> Waker {
    unsafe { Waker::from_raw(glib_raw_waker(ctx)) }
}

fn glib_raw_waker(ctx: MainContext) -> RawWaker {
    let ctx: *mut GMainContext = ctx.into_glib_ptr();
    RawWaker::new(
        ctx.cast(),
        &RawWakerVTable::new(
            glib_waker_clone,
            glib_waker_wake,
            glib_waker_wake_by_ref,
            glib_waker_drop,
        ),
    )
}

unsafe fn glib_waker_clone(ctx: *const ()) -> RawWaker {
    let ctx = ctx.cast::<GMainContext>().cast_mut();
    let ctx = unsafe { MainContext::from_glib_borrow(ctx) };
    let ctx = ctx.clone();
    glib_raw_waker(ctx)
}

unsafe fn glib_waker_wake(ctx: *const ()) {
    let ctx = ctx.cast::<GMainContext>().cast_mut();
    let ctx = unsafe { MainContext::from_glib_full(ctx) };
    ctx.wakeup();
}

unsafe fn glib_waker_wake_by_ref(ctx: *const ()) {
    let ctx = ctx.cast::<GMainContext>().cast_mut();
    let ctx = unsafe { MainContext::from_glib_borrow(ctx) };
    ctx.wakeup();
}

unsafe fn glib_waker_drop(ctx: *const ()) {
    let ctx = ctx.cast::<GMainContext>().cast_mut();
    let _ = unsafe { MainContext::from_glib_full(ctx) };
}
