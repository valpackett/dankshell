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
use protos::{gtkclient, permissions};

fn main() {
    pretty_env_logger::init();
    gtk::init().expect("gtk::init");

    let ((mut layer_shell, mut dank_private), _lshthread) = gtkclient::get_globals(|globals| {
        (
            gtkclient::get_layer_shell(globals),
            gtkclient::get_dank_private(globals),
        )
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

    button.connect_clicked(move |_| {
        println!("Clicked!");
        layer_surface.set_margin(0, 0, 0, 0);
        layer_surface.set_anchor(Anchor::Bottom | Anchor::Top | Anchor::Right);
        layer_surface.set_size(64, 0);
        use gtkclient::api::RequestsTrait;
        dank_private.spawn_program("dankshell-shell-experience".to_owned(), Some(permissions::Permissions {
            layer_shell: permissions::LayerShellPermissions {
                background: true,
                bottom: true,
                top: true,
                overlay: false,
            },
            private_api: true,
        }.to_cbor().unwrap()));
        dank_private.spawn_program("weston-terminal".to_owned(), None);
    });

    gtk::main();
}
