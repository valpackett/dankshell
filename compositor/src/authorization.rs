use std::ptr;
use std::os::unix::io::RawFd;
use nix;
use nix::fcntl::{self, FdFlag, FcntlArg};
use nix::sys::socket::{socketpair, AddressFamily, SockFlag, SockType};
use weston_rs::Display;
use weston_rs::wayland_server::{Resource, Client};
use weston_rs::wayland_server::commons::Interface;
pub use protos::permissions::*;

pub fn resource_client_permissions<'a, T: Interface>(res: &'a Resource<T>) -> Option<&'a mut Permissions> {
    if let Some(client) = res.client() {
        let ud = client.get_user_data();
        if ud == ptr::null_mut() {
            return None;
        }
        Some(unsafe { &mut *(ud as *mut Permissions) })
    } else {
        warn!("request from dead client");
        None
    }
}

/// Makes a socket pair and attaches the server end to the Wayland server, setting user data to the
/// permissions object.
///
/// Returns the *client* socket that should be passed to the client as `WAYLAND_SOCKET` and closed
/// in the current process after passing.
pub fn start_client_socket_with_permissions(display: &mut Display, ps: Permissions) -> nix::Result<RawFd> {
    let (sock_server, sock_client) = socketpair(AddressFamily::Unix, SockType::Stream, None, SockFlag::empty())?;
    let _ = fcntl::fcntl(sock_server, FcntlArg::F_SETFD(FdFlag::FD_CLOEXEC))?; // not like we're going to exec but still
    let client = unsafe { display.create_client(sock_server) };
    client.set_user_data(Box::into_raw(Box::new(ps)) as *mut _);
    client.set_destructor(destroy_user_data);
    Ok(sock_client)
}

fn destroy_user_data(ps: *mut ()) {
    unsafe { Box::from_raw(ps as *mut Permissions) };
}
