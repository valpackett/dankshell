#![feature(proc_macro)]

extern crate libc;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate wayland_client;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
extern crate protos;

mod panel;

use protos::gtkclient;

fn main() {
    pretty_env_logger::init();
    gtk::init().expect("gtk::init");

    let ((layer_shell, dank_private), _lshthread) = gtkclient::get_globals(|globals| {
        (
            gtkclient::get_layer_shell(globals),
            gtkclient::get_dank_private(globals),
        )
    });

    let panel = relm::init::<panel::Panel>((layer_shell, dank_private)).expect("init Panel");

    panel.emit(panel::Msg::Reconfigure(vec![
        panel::WidgetConfig::Clock(Default::default()),
        panel::WidgetConfig::QuickLaunch(Default::default()),
    ]));

    gtk::main();
}
