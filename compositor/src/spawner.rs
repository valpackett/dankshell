use libc;
use std::{env, thread, time, ffi};
use std::process::Command;
use std::os::unix::io::RawFd;
use std::os::unix::process::CommandExt;
use log::*;
use serde_derive::{Serialize, Deserialize};
use pdfork::*;
use loginw::priority;
use weston_rs::Display;
use tiny_nix_ipc::Socket;
use crate::authorization::{self, Permissions};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    SetDisplayName(ffi::OsString),
    Spawn(String),
}

pub fn start_spawner() -> (ChildHandle, Socket) {
    let (sock_parent, sock_child) = Socket::new_socketpair().unwrap();
    match fork() {
        ForkResult::Fail => panic!("fork"),
        ForkResult::Parent(child_proc) => {
            drop(sock_child);
            (child_proc, sock_parent)
        },
        ForkResult::Child => {
            drop(sock_parent);
            spawner_loop(sock_child);
        }
    }
}

fn spawner_loop(mut sock: Socket) -> ! {
    let mut wl_disp = None;
    loop {
        use self::Request::*;
        // TODO check that peer is alive
        match sock.recv_cbor::<Request, [RawFd; 1]>(1024) {
            Ok((SetDisplayName(name), _)) => {
                wl_disp = Some(name);
            },
            Ok((Spawn(prog), fds)) => {
                let disp = wl_disp.clone().expect("WAYLAND_DISPLAY must have been set");
                let prog1 = prog.clone();
                if let Err(err) = Command::new(&prog).before_exec(move || {
                    // loginw sets realtime priority for the compositor
                    // see https://blog.martin-graesslin.com/blog/2017/09/kwinwayland-goes-real-time/ for reasons
                    // we obviously don't want it in user applications :D
                    priority::make_normal();
                    env::remove_var("DISPLAY");
                    if let Some([fd]) = fds {
                        info!("Spawning '{}' with WAYLAND_SOCKET={}", prog1, fd);
                        env::remove_var("WAYLAND_DISPLAY");
                        env::set_var("WAYLAND_SOCKET", format!("{}", fd));
                    } else {
                        info!("Spawning '{}' with WAYLAND_DISPLAY={:?}", prog1, disp);
                        env::set_var("WAYLAND_DISPLAY", &disp);
                    }
                    Ok(())
                }).spawn() {
                    warn!("Failed to spawn '{}': {:?}", prog, err);
                }
                if let Some([fd]) = fds {
                    unsafe { libc::close(fd) };
                }
            },
            Err(err) => {
                warn!("Failed to recv: {:?}", err);
                thread::sleep(time::Duration::from_millis(1000));
            }
        }
    }
}

pub fn spawn(display: &mut Display, spawner_sock: &mut Socket, cmd: &str, ps: Option<Permissions>) {
    if let Some(ps) = ps {
        let sock = authorization::start_client_socket_with_permissions(display, ps).unwrap();
        let _ = spawner_sock.send_cbor(&Request::Spawn(cmd.to_owned()), Some(&[sock]));
        unsafe { libc::close(sock) };
    } else {
        let _ = spawner_sock.send_cbor(&Request::Spawn(cmd.to_owned()), None);
    }
}
