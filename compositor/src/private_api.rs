use std::{mem};
use std::os::unix::io::{AsRawFd, FromRawFd};
use log::*;
use parking_lot::RwLock;
use weston_rs::CompositorRef;
use wayland_server::{NewResource, Resource, Display, LoopToken};
use wayland_server::commons::Implementation;
use tiny_nix_ipc::Socket;
use protos::{
    CborConv,
    outputs::*,
    dank_private::server::dank_shell as api,
};
use crate::authorization::{self, Permissions};
use crate::{util, spawner, COMPOSITOR};

// There should only be one shell experience running, but it's easier to support multiple lol
static PAPI_RESOURCES: RwLock<Vec<Resource<api::DankShell>>> = RwLock::new(Vec::new());

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
                        CborConv::from_cbor(&ps).map_err(|e| {
                            warn!("CBOR decoding permissions error: {:?}", e);
                            e
                        }).ok()
                    })
                );
            }
        }
    }
}

pub fn send_output_info(compositor: &mut CompositorRef) {
    let heads = compositor.iterate_heads().map(|head| HeadInfo {
        name: util::cstr_to_string(head.get_name()),
        make: util::opt_cstr_to_string(head.get_make()),
        model: util::opt_cstr_to_string(head.get_model()),
        serial_number: util::opt_cstr_to_string(head.get_serial_number()),
        mm_width: head.mm_width(),
        mm_height: head.mm_height(),
        internal: head.connection_internal(),
        connected: head.is_connected(),
        enabled: head.is_enabled(),
        device_changed: head.is_device_changed(),
        output: head.output().map(|output| OutputInfo {
            id: output.id(),
            transform: output.transform(),
            scale: output.scale(),
            extra_scale: output.extra_scale(),
            x: output.x(),
            y: output.y(),
            width: output.width(),
            height: output.height(),
        })
    }).collect::<Vec<_>>();
    if let Ok(data) = (OutputState { heads }).to_cbor() {
        for res in PAPI_RESOURCES.write().iter() {
            res.send(api::Event::OutputState { data: data.clone() });
        }
    } else {
        error!("Could not encode OutputState");
    }
}

pub fn register_private_api(display: &mut Display, token: &LoopToken, socket: &Socket) {
    let dptr = display as *mut Display; // It's defined at the start of main and lives till the end
    let sfd = socket.as_raw_fd();
    display.create_global::<api::DankShell, _>(&token, 1, move |_, res: NewResource<api::DankShell>| {
        let r = res.implement(PrivateApiImpl {
            display: dptr,
            socket: mem::ManuallyDrop::new(unsafe { Socket::from_raw_fd(sfd) }),
        }, Some(|_, _| {}));
        {
            PAPI_RESOURCES.write().push(r);
        }
        send_output_info(&mut COMPOSITOR.write());
    });
}
