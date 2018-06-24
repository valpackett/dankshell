#![feature(proc_macro)]
#![feature(const_fn)]
#![feature(const_vec_new)]
#![feature(fnbox)]

extern crate libc;
extern crate gtk;
extern crate glib;
extern crate fragile;
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
extern crate ini;
extern crate glob;
extern crate chrono;
extern crate wayland_client;
extern crate atomicwrites;
extern crate parking_lot;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate protos;

mod conf;
mod desktop_entries;
mod panel;
mod launcher;

use std::rc::Rc;
use fragile::Sticky;
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

    let (desk_tx, _deskthread) = desktop_entries::spawn_reader();

    let launcher = Rc::new(relm::init::<launcher::Launcher>(
        (layer_shell.clone(), dank_private.clone())
    ).expect("init Launcher"));

    let launcher_d = Sticky::new(Rc::clone(&launcher));
    desk_tx.send(Box::new(move || {
        let _ = glib::idle_add(move || {
            launcher_d.get().emit(launcher::Msg::ReloadApps);
            glib::Continue(false)
        });
    })).unwrap();

    let panel = relm::init::<panel::Panel>(
        (layer_shell, dank_private, Rc::clone(&launcher))
    ).expect("init Panel");

    panel.emit(panel::Msg::Reconfigure(confmgr.read("panel.ron").expect("panel config read")));
    launcher.emit(launcher::Msg::Reconfigure(Default::default()));

    gtk::main();
}
