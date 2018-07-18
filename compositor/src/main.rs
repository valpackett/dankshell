#![feature(rust_2018_preview, use_extern_macros, nll, vec_remove_item, const_fn, const_vec_new)]

mod util;
mod spawner;
mod authorization;
mod surface_registry;
mod backend;
mod ctx;
mod moove;
mod resize;
mod desktop;
mod focus;
mod layer_shell;
mod private_api;

use log::*;
use weston_rs::*;
use crate::util::MutStatic;
use crate::ctx::SurfaceContext;
use crate::authorization::{Permissions, LayerShellPermissions};

pub static COMPOSITOR: MutStatic<Compositor> = MutStatic::new();
pub static DESKTOP: MutStatic<Desktop<SurfaceContext>> = MutStatic::new();

weston_logger!{fn wlog(msg: &str) {
    info!(target: "weston", "{}", msg);
}}

fn main() {
    pretty_env_logger::init();
    weston_rs::log_set_handler(wlog, wlog);

    let (_child_proc, mut spawner_sock) = spawner::start_spawner();

    let (mut display, mut event_loop) = Display::new();
    let mut compositor = Compositor::new(&display, &mut event_loop);

    compositor.set_xkb_rule_names(None); // defaults to environment variables

    // Make a socket for clients to connect to
    let sock_name = display.add_socket_auto().expect("add_socket_auto");
    spawner_sock.send_cbor(&spawner::Request::SetDisplayName(sock_name), None).unwrap();

    // Backend/head/output setup
    let be = backend::start_backend(&mut compositor, &mut event_loop);
    compositor.add_heads_changed_listener(backend::heads_changed_listener(be));
    compositor.flush_heads_changed();

    // Sandbox the process if available on the OS (e.g. FreeBSD Capsicum).
    // Nothing should need FS access from this point on.
    // (Well, dynamic reconfiguration of XKB probably will...)
    rusty_sandbox::Sandbox::new().sandbox_this_process();

    // Background color
    let mut bg_layer = Layer::new(&compositor);
    bg_layer.set_position(POSITION_BACKGROUND);
    let mut bg_surf = Surface::new(&compositor);
    bg_surf.set_size(8096, 8096);
    bg_surf.set_color(0.1, 0.3, 0.6, 1.0);
    let bg_view = View::new(&bg_surf);
    bg_layer.view_list_entry_insert(&bg_view);

    // Our data for libweston-desktop stuff
    let desktop_impl = Box::new(desktop::DesktopImpl::new(&compositor));

    // The libweston-desktop object
    // NOTE: Important to keep around (do not do 'let _')
    let desktop = Desktop::new(&compositor, desktop_impl);

    // Left click to focus window
    compositor.add_button_binding(ev::BTN_LEFT, KeyboardModifier::empty(), &|p, _, _| focus::click_activate(p));
    // Right click to focus window
    compositor.add_button_binding(ev::BTN_RIGHT, KeyboardModifier::empty(), &|p, _, _| focus::click_activate(p));

    focus::keyboard_focus_listener().signal_add(
        compositor.first_seat().expect("first_seat")
        .keyboard().expect("first_seat keyboard")
        .focus_signal());

    // Ctrl+Enter to spawn a terminal
    compositor.add_key_binding(ev::KEY_ENTER, KeyboardModifier::CTRL, &|_, _, _| {
        let _ = spawner_sock.send_cbor(&spawner::Request::Spawn("weston-terminal".to_owned()), None);
    });

    // Setup layer-shell
    layer_shell::create_layers(&compositor);
    layer_shell::register_layer_shell(&mut display, &event_loop.token());

    // Setup private API (for shell-experience)
    private_api::register_private_api(&mut display, &event_loop.token(), &spawner_sock);

    // Go!
    compositor.wake();
    COMPOSITOR.set(compositor);
    DESKTOP.set(desktop);
    spawner::spawn(&mut display, &mut spawner_sock, "dankshell-shell-experience", Some(Permissions {
        layer_shell: LayerShellPermissions {
            background: true,
            bottom: true,
            top: true,
            overlay: false,
        },
        private_api: true,
    }));
    let _ = event_loop.run();
}
