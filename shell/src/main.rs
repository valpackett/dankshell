#![cfg_attr(feature = "cargo-clippy", allow(redundant_field_names, let_and_return))] // relm-attributes
#![cfg_attr(feature = "cargo-clippy", allow(identity_op, const_static_lifetime))] // serde-derive
#![feature(rust_2018_preview, use_extern_macros, const_fn, const_vec_new)]

#[macro_use]
extern crate error_chain;

mod evf;
mod conf;
mod panel;
mod launcher;
mod settings;

use std::rc::Rc;
use log::*;
use wayland_client::{Proxy, commons::Implementation};
use protos::{
    CborConv,
    gtkclient,
    outputs,
    dank_private::client::dank_shell as api,
};

fn send_msg<D, M>(data: &[u8], constr: fn(D) -> M, sender: fn(M)) where D: CborConv {
    match D::from_cbor(&data) {
        Ok(s) => sender(constr(s)),
        Err(e) => error!("CBOR decoding error: {:?}", e),
    }
}

fn main() {
    pretty_env_logger::init();
    gtk::init().expect("gtk::init");

    let ((layer_shell, dank_private), _lshthread) = gtkclient::get_globals(|globals| {
        (
            gtkclient::get_layer_shell(globals),
            gtkclient::get_dank_private(globals, |msg, _| {
                use self::api::Event::*;
                match msg {
                    OutputState { data } => send_msg(&data, settings::Msg::NextOutputState, evf::send_to_settings),
                }
            }),
        )
    });

    let confmgr = conf::ConfigManager::new();

    let launcher = Rc::new(relm::init::<launcher::Launcher>(
        (layer_shell.clone(), dank_private.clone())
    ).expect("init Launcher"));

    let panel = relm::init::<panel::Panel>(
        (layer_shell, dank_private.clone(), Rc::clone(&launcher))
    ).expect("init Panel");

    let settings = relm::init::<settings::Settings>(
        dank_private
    ).expect("init Settings");

    panel.emit(panel::Msg::Reconfigure(confmgr.read("panel.ron").expect("panel config read")));
    launcher.emit(launcher::Msg::Reconfigure(Default::default()));
    launcher.emit(launcher::Msg::ReloadApps);

    evf::set_settings(settings);

    gtk::main();
}
