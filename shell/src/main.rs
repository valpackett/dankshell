#![cfg_attr(feature = "cargo-clippy", allow(redundant_field_names, let_and_return))] // relm-attributes
#![cfg_attr(feature = "cargo-clippy", allow(identity_op, const_static_lifetime))] // serde-derive
#![feature(rust_2018_preview, proc_macro, const_fn, const_vec_new, fnbox)]

#[macro_use]
extern crate error_chain;

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
