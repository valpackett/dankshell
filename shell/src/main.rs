extern crate libc;
extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate send_cell;
extern crate wayland_client;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate protos;

use gtk::prelude::*;
use gtk::{Button, Window, WindowType};
use protos::gtkclient;

fn main() {
    pretty_env_logger::init();
    gtk::init().expect("gtk::init");

    let (mut layer_shell, _lshthread) = gtkclient::get_globals(|globals| {
        gtkclient::get_layer_shell(globals)
    });

    let mut window = Window::new(WindowType::Toplevel);
    window.set_title("test");
    window.set_default_size(320, 24);
    window.set_decorated(false);
    let button = Button::new_with_label("Click me!");
    window.add(&button);

    use gtkclient::lsr::{Anchor, RequestsTrait};
    let layer_surface = gtkclient::get_layer_surface(&mut layer_shell, &mut window, gtkclient::lsh::Layer::Top);
    layer_surface.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right);
    layer_surface.set_margin(10, 10, 10, 10);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    button.connect_clicked(move |_| {
        println!("Clicked!");
        layer_surface.set_margin(0, 0, 0, 0);
        layer_surface.set_anchor(Anchor::Bottom | Anchor::Top | Anchor::Right);
        layer_surface.set_size(64, 0);
    });

    gtk::main();
}
