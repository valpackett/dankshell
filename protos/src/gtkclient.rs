use {libc, gdk, glib};
use std::{thread, sync};
use std::sync::Arc;
use gtk::{Window, WidgetExt, GtkWindowExt, Continue};
use glib::translate::ToGlibPtr;
use gdk_sys::{GdkDisplay, GdkWindow};
use fragile::Sticky;
use wayland_client::{Display, GlobalManager, Proxy};
use wayland_client::commons::Implementation;
use wayland_client::protocol::wl_display::RequestsTrait;
use wayland_client::sys::client::wl_display;
pub use layer_shell::client::zxdg_layer_shell_v1 as lsh;
pub use layer_shell::client::zxdg_layer_surface_v1 as lsr;
pub use dank_private::client::dank_shell as api;

#[allow(non_camel_case_types)]
type wl_surface = libc::c_void;

struct UnsafeSendWrapper<T>(T);
unsafe impl<T> Send for UnsafeSendWrapper<T> {}

struct UnsafeSyncWrapper<T>(T);
unsafe impl<T> Sync for UnsafeSyncWrapper<T> {}

extern "C" {
    fn gdk_wayland_display_get_wl_display(display: *mut GdkDisplay) -> *mut wl_display;
    fn gdk_wayland_window_set_use_custom_surface(window: *mut GdkWindow);
    fn gdk_wayland_window_get_wl_surface(window: *mut GdkWindow) -> *mut wl_surface;
}

pub type LayerShellApi = Proxy<lsh::ZxdgLayerShellV1>;
pub type LayerSurfaceApi = Proxy<lsr::ZxdgLayerSurfaceV1>;
pub type DankShellApi = Proxy<api::DankShell>;

/// Get proxies for whatever globals you want from the Wayland display GDK is connected to.
///
/// Also spawns a thread that polls the Wayland event queue wayland-rs has created
/// to make sure the events for these globals actually get dispatched.
/// TODO: wayland-rs should provide a method that doesn't create a new event queue?
pub fn get_globals<T: Send + 'static>(global_getter: fn(&GlobalManager) -> T) -> (T, thread::JoinHandle<()>) {
    let gdk_display = gdk::Display::get_default();
    let wl_display = UnsafeSendWrapper(unsafe { gdk_wayland_display_get_wl_display(gdk_display.to_glib_none().0) });
    let (tx, rx) = sync::mpsc::channel();
    let thread = thread::Builder::new().name("layer-shell wl event queue poller".to_owned()).spawn(move || {
        let (display, mut event_queue) = unsafe { Display::from_external_display(wl_display.0) };
        let globals = GlobalManager::new(display.get_registry().unwrap());
        event_queue.sync_roundtrip().expect("wayland roundtrip");
        for (id, interface, version) in globals.list() {
            debug!("wl global {}: {} (version {})", id, interface, version);
        }
        tx.send(global_getter(&globals)).unwrap();
        loop {
            event_queue.dispatch().expect("layer-shell event queue dispatch");
        }
    }).unwrap();
    (rx.recv().unwrap(), thread)
}

pub fn get_layer_shell(globals: &GlobalManager) -> LayerShellApi {
    globals.instantiate_auto::<lsh::ZxdgLayerShellV1>()
        .expect("xdg-layer-shell protocol from compositor")
        .implement(|_, _| {
            warn!("layer-shell event (wtf?)");
        })
}

pub fn get_layer_surface(layer_shell: &mut LayerShellApi, window: &mut Window, layer: lsh::Layer) -> LayerSurfaceApi {
    use self::lsh::RequestsTrait;
    window.realize();
    let gdk_window_ptr = window.get_window().expect("window").to_glib_none().0;
    unsafe { gdk_wayland_window_set_use_custom_surface(gdk_window_ptr) };
    let wl_surface = unsafe { gdk_wayland_window_get_wl_surface(gdk_window_ptr) };
    layer_shell.get_layer_surface(
        &unsafe { Proxy::from_c_ptr(wl_surface as *mut _) },
        None, layer, "".to_owned()
        ).expect("get_layer_surface")
        .implement(LayerSurfaceImpl { window: Arc::new(UnsafeSyncWrapper(Sticky::new(window.clone()))) })
}

struct LayerSurfaceImpl {
    window: Arc<UnsafeSyncWrapper<Sticky<Window>>>
}

impl Implementation<Proxy<lsr::ZxdgLayerSurfaceV1>, lsr::Event> for LayerSurfaceImpl {
    fn receive(&mut self, msg: lsr::Event, _: Proxy<lsr::ZxdgLayerSurfaceV1>) {
        if let lsr::Event::Configure { serial, width, height } = msg {
            let wnd = Arc::clone(&self.window);
            glib::idle_add(move || {
                debug!("layer-shell configure event {:?}: {}x{}", serial, width, height);
                wnd.0.get().resize(width as i32, height as i32);
                Continue(false)
            });
        }
    }
}


pub fn get_dank_private(globals: &GlobalManager) -> DankShellApi {
    globals.instantiate_auto::<api::DankShell>()
        .expect("dankshell private protocol from compositor")
        .implement(|_, _| {
            warn!("dank event (wtf?)");
        })
}
