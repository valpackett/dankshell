use std::mem;
use std::os::unix::io::{AsRawFd, FromRawFd};
use wayland_server::{NewResource, Resource, Display, LoopToken};
use wayland_server::commons::Implementation;
use tiny_nix_ipc::Socket;
use protos::dank_private::server::dank_shell as api;
use authorization::{self, Permissions};
use spawner;

struct PrivateApiImpl {
    display: *mut Display,
    socket: mem::ManuallyDrop<Socket>,
}

unsafe impl Send for PrivateApiImpl {}

impl Implementation<Resource<api::DankShell>, api::Request> for PrivateApiImpl {
    fn receive(&mut self, msg: api::Request, resource: Resource<api::DankShell>) {
        if let Some(Permissions { private_api, .. }) = authorization::resource_client_permissions(&resource) {
            if !*private_api {
                warn!("Private API not allowed");
                return
            }
        } else {
            warn!("No permissions found");
            return
        };
        use self::api::Request::*;
        match msg {
            SpawnProgram { command, permissions } => {
                spawner::spawn(
                    unsafe { &mut *self.display }, &mut *self.socket, &command,
                    permissions.and_then(|ps| {
                        if ps.is_empty() {
                            return None
                        }
                        Permissions::from_cbor(&ps).map_err(|e| {
                            warn!("CBOR decoding permissions error: {:?}", e);
                            e
                        }).ok()
                    })
                );
            }
        }
    }
}

pub fn register_private_api(display: &mut Display, token: &LoopToken, socket: &Socket) {
    let dptr = display as *mut Display; // It's defined at the start of main and lives till the end
    let sfd = socket.as_raw_fd();
    display.create_global::<api::DankShell, _>(&token, 1, move |_, res: NewResource<api::DankShell>| {
        res.implement(PrivateApiImpl {
            display: dptr,
            socket: mem::ManuallyDrop::new(unsafe { Socket::from_raw_fd(sfd) }),
        }, Some(|_, _| {}));
    });
}
