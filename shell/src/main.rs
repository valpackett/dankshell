#![feature(proc_macro)]

extern crate libc;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate ron;
extern crate xdg;
extern crate chrono;
extern crate wayland_client;
extern crate atomicwrites;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate protos;

mod conf;
mod panel;
mod launcher;

use std::rc::Rc;
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

    let confmgr = conf::ConfigManager::new();

    let launcher = Rc::new(relm::init::<launcher::Launcher>(
        (layer_shell.clone(), dank_private.clone())
    ).expect("init Launcher"));

    let panel = relm::init::<panel::Panel>(
        (layer_shell, dank_private, Rc::clone(&launcher))
    ).expect("init Panel");

    panel.emit(panel::Msg::Reconfigure(confmgr.read("panel.ron").expect("panel config read")));
    launcher.emit(launcher::Msg::Reconfigure(Default::default()));
    launcher.emit(launcher::Msg::Hide);

    gtk::main();
}
