use std::{
    cell::RefCell,
    rc::Rc,
    task::{RawWaker, RawWakerVTable, Waker},
};

use dispatch2::DispatchQueue;
use futures_util::FutureExt;
use objc2::{
    ClassType, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_foundation::{NSObject, NSObjectProtocol, ns_string};
use objc2_ui_kit::{
    UIApplication, UIApplicationDelegate, UICoordinateSpace, UIScene, UISceneConfiguration,
    UISceneConnectionOptions, UISceneDelegate, UISceneSession, UIWindowScene,
    UIWindowSceneDelegate, UIWindowSceneGeometry,
};
use slab::Slab;
use winio_callback::{Callback, Runnable};

use crate::{Error, Result};

pub struct App {
    mtm: MainThreadMarker,
}

impl App {
    pub fn new() -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
        Ok(Self { mtm })
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        let future = future.map(|_| {
            std::process::exit(0);
        });
        winio_pollable::enter_block_on(future, dispatcher_waker(), || {
            crate::catch(|| {
                // Register the class.
                let _ = AppDelegate::new(self.mtm);
                UIApplication::main(None, Some(ns_string!(AppDelegate::NAME)), self.mtm);
            })
            .unwrap()
        })
    }
}

fn dispatcher_waker() -> Waker {
    unsafe { Waker::from_raw(dispatcher_raw_waker()) }
}

fn dispatcher_raw_waker() -> RawWaker {
    RawWaker::new(
        std::ptr::null(),
        &RawWakerVTable::new(
            dispatcher_clone,
            dispatcher_wake,
            dispatcher_wake_by_ref,
            dispatcher_drop,
        ),
    )
}

unsafe fn dispatcher_clone(_: *const ()) -> RawWaker {
    dispatcher_raw_waker()
}

unsafe fn dispatcher_wake(data: *const ()) {
    unsafe { dispatcher_wake_by_ref(data) }
}

unsafe fn dispatcher_wake_by_ref(_: *const ()) {
    DispatchQueue::main().exec_async(|| {
        winio_pollable::run_current_task();
    })
}

unsafe fn dispatcher_drop(_: *const ()) {}

thread_local! {
    pub(crate) static RESIZE_SLAB: RefCell<Slab<Rc<Callback>>> = const { RefCell::new(Slab::new()) };
    pub(crate) static MOVE_SLAB: RefCell<Slab<Rc<Callback>>> = const { RefCell::new(Slab::new()) };
}

fn signal_resize<R: Runnable>() {
    RESIZE_SLAB.with_borrow(|s| {
        for (_, callback) in s.iter() {
            callback.signal::<R>(());
        }
    });
}

fn signal_move<R: Runnable>() {
    MOVE_SLAB.with_borrow(|s| {
        for (_, callback) in s.iter() {
            callback.signal::<R>(());
        }
    });
}

#[derive(Debug)]
struct AppDelegateIvars {}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "AppDelegate"]
    #[ivars = AppDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct AppDelegate;

    impl AppDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(AppDelegateIvars {});
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for AppDelegate {}

    #[allow(non_snake_case)]
    unsafe impl UIApplicationDelegate for AppDelegate {
        #[unsafe(method_id(application:configurationForConnectingSceneSession:options:))]
        fn application_configurationForConnectingSceneSession_options(
            &self,
            application: &UIApplication,
            connecting_scene_session: &UISceneSession,
            options: &UISceneConnectionOptions,
        ) -> Retained<UISceneConfiguration> {
            let scene_config = UISceneConfiguration::initWithName_sessionRole(
                UISceneConfiguration::alloc(self.mtm()),
                Some(ns_string!("Default Configuration")),
                &connecting_scene_session.role()
            );
            unsafe { scene_config.setDelegateClass(Some(AppDelegate::class())) };
            scene_config
        }
    }

    #[allow(non_snake_case)]
    unsafe impl UISceneDelegate for AppDelegate {
        #[unsafe(method(scene:willConnectToSession:options:))]
        fn scene_willConnectToSession_options(
            &self,
            scene: &UIScene,
            session: &UISceneSession,
            connection_options: &UISceneConnectionOptions,
        ) {
            winio_pollable::run_current_task();
        }

        #[unsafe(method(sceneDidBecomeActive:))]
        fn sceneDidBecomeActive(&self, scene: &UIScene) {
            signal_resize::<()>();
            winio_pollable::run_current_task();
        }
    }

    #[allow(non_snake_case)]
    unsafe impl UIWindowSceneDelegate for AppDelegate {
        #[unsafe(method(windowScene:didUpdateEffectiveGeometry:))]
        fn windowScene_didUpdateEffectiveGeometry(
            &self,
            window_scene: &UIWindowScene,
            previous_effective_geometry: Option<&UIWindowSceneGeometry>,
        ) {
            let geometry = window_scene.effectiveGeometry();
            let bounds = geometry.coordinateSpace(self.mtm()).bounds();
            let previous_bounds = previous_effective_geometry.map(|g| g.coordinateSpace(self.mtm()).bounds());
            if Some(bounds.size) != previous_bounds.map(|b| b.size) {
                signal_resize::<()>();
            }
            if Some(bounds.origin) != previous_bounds.map(|b| b.origin) {
                signal_move::<()>();
            }
            winio_pollable::run_current_task();
        }
    }
}

impl AppDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc(), init] }
    }
}
