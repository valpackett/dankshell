use {libc, gdk};
use gtk::WidgetExt;
use glib::translate::ToGlibPtr;
use gdk_sys::{GdkDisplay, GdkWindow};
use wayland_client::{Display, GlobalManager, Proxy};
use wayland_client::protocol::wl_display::RequestsTrait;
use wayland_client::sys::client::wl_display;
pub use protos::layer_shell::client::zxdg_layer_shell_v1 as lsh;
pub use protos::layer_shell::client::zxdg_layer_surface_v1 as lsr;

#[allow(non_camel_case_types)]
type wl_surface = libc::c_void;

extern "C" {
    fn gdk_wayland_display_get_wl_display(display: *mut GdkDisplay) -> *mut wl_display;
    fn gdk_wayland_window_set_use_custom_surface(window: *mut GdkWindow);
    fn gdk_wayland_window_get_wl_surface(window: *mut GdkWindow) -> *mut wl_surface;
}

pub type LayerShellApi = Proxy<lsh::ZxdgLayerShellV1>;
pub type LayerSurfaceApi = Proxy<lsr::ZxdgLayerSurfaceV1>;

pub fn get_layer_shell() -> LayerShellApi {
    let gdk_display = gdk::Display::get_default();
    let (display, mut event_queue) = unsafe { Display::from_external_display(
            gdk_wayland_display_get_wl_display(gdk_display.to_glib_none().0)) };
    let globals = GlobalManager::new(display.get_registry().unwrap());
    event_queue.sync_roundtrip().expect("wayland roundtrip");
    for (id, interface, version) in globals.list() {
        debug!("wl global {}: {} (version {})", id, interface, version);
    }
    globals.instantiate_auto::<lsh::ZxdgLayerShellV1>()
        .expect("xdg-layer-shell protocol from compositor")
        .implement(|_, _| {})
}

pub fn get_layer_surface(layer_shell: &mut LayerShellApi, window: &mut impl WidgetExt) -> LayerSurfaceApi {
    use self::lsh::RequestsTrait;
    window.realize();
    let gdk_window_ptr = window.get_window().expect("window").to_glib_none().0;
    unsafe { gdk_wayland_window_set_use_custom_surface(gdk_window_ptr) };
    let wl_surface = unsafe { gdk_wayland_window_get_wl_surface(gdk_window_ptr) };
    layer_shell.get_layer_surface(
        &unsafe { Proxy::from_c_ptr(wl_surface as *mut _) },
        None, lsh::Layer::Top, "".to_owned()
        ).expect("get_layer_surface")
        .implement(|_, _| {})
}
