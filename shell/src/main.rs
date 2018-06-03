extern crate libc;
extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate wayland_client;
extern crate protos;

use gtk::prelude::*;
use gtk::{Button, Window, WindowType};
use glib::translate::ToGlibPtr;
use wayland_client::{Display, GlobalManager, Proxy};
use wayland_client::protocol::wl_display::RequestsTrait;
use wayland_client::sys::client::wl_display;
use protos::layer_shell::client::zxdg_layer_shell_v1 as lsh;
use lsh::RequestsTrait as LRequestsTrait;

#[allow(non_camel_case_types)]
type wl_surface = libc::c_void;

extern "C" {
    fn gdk_wayland_display_get_wl_display(display: *mut gdk_sys::GdkDisplay) -> *mut wl_display;
    fn gdk_wayland_window_set_use_custom_surface(window: *mut gdk_sys::GdkWindow);
    fn gdk_wayland_window_get_wl_surface(window: *mut gdk_sys::GdkWindow) -> *mut wl_surface;
}

fn main() {
    gtk::init().expect("gtk::init");

    let gdk_display = gdk::Display::get_default();
    let (display, mut event_queue) = unsafe { Display::make_display(gdk_wayland_display_get_wl_display(gdk_display.to_glib_none().0)) }.expect("make_display");
    let globals = GlobalManager::new(display.get_registry().unwrap());
    event_queue.sync_roundtrip().unwrap();
    for (id, interface, version) in globals.list() {
        println!("{}: {} (version {})", id, interface, version);
    }
    let layer_shell = globals.instantiate_auto::<lsh::ZxdgLayerShellV1>().expect("xdg-layer-shell protocol from compositor").implement(|_, _| {});

    let window = Window::new(WindowType::Toplevel);
    window.set_title("test");
    window.set_default_size(1280, 24);
    window.set_decorated(false);
    let button = Button::new_with_label("Click me!");
    window.add(&button);

    window.realize();
    let gdk_window_ptr = window.get_window().expect("window").to_glib_none().0;
    unsafe { gdk_wayland_window_set_use_custom_surface(gdk_window_ptr) };
    let wl_surface = unsafe { gdk_wayland_window_get_wl_surface(gdk_window_ptr) };
    let layer_surface = layer_shell.get_layer_surface(&unsafe { Proxy::from_c_ptr(wl_surface as *mut _) },
        None, lsh::Layer::Top, "test".to_owned()).expect("get_layer_surface");
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    button.connect_clicked(|_| {
        println!("Clicked!");
    });

    gtk::main();
}
