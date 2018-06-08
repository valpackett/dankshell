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

mod layer_shell;

fn main() {
    pretty_env_logger::init();
    gtk::init().expect("gtk::init");

    let (mut layer_shell, _lshthread) = layer_shell::get_layer_shell();

    let mut window = Window::new(WindowType::Toplevel);
    window.set_title("test");
    window.set_default_size(320, 24);
    window.set_decorated(false);
    let button = Button::new_with_label("Click me!");
    window.add(&button);

    use layer_shell::lsr::{Anchor, RequestsTrait};
    let layer_surface = layer_shell::get_layer_surface(&mut layer_shell, &mut window);
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
