use libc;
use std::{env, thread, time, ffi};
use std::process::Command;
use std::os::unix::process::CommandExt;
use pdfork::*;
use loginw::priority;
use ipc_channel::ipc::{self, IpcOneShotServer, IpcSender, IpcReceiver};

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    SetDisplayName(ffi::OsString),
    Spawn(String),
}

pub fn start_spawner() -> (ChildHandle, IpcSender<Request>) {
    let (server0, server0_name) = IpcOneShotServer::<Request>::new().unwrap();
    match fork() {
        ForkResult::Fail => panic!("fork"),
        ForkResult::Parent(child_proc) => {
            let tx = IpcSender::connect(server0_name).unwrap();
            (child_proc, tx)
        },
        ForkResult::Child => {
            let (rx, _) = server0.accept().unwrap();
            spawner_loop(rx);
        }
    }
}

fn spawner_loop(rx: IpcReceiver<Request>) -> ! {
    let mut wl_disp = None;
    loop {
        use self::Request::*;
        match rx.recv() {
            Ok(SetDisplayName(name)) => {
                wl_disp = Some(name);
            },
            Ok(Spawn(prog)) => {
                let disp = wl_disp.clone().expect("WAYLAND_DISPLAY must have been set");
                if let Err(err) = Command::new(&prog).before_exec(move || {
                    // loginw sets realtime priority for the compositor
                    // see https://blog.martin-graesslin.com/blog/2017/09/kwinwayland-goes-real-time/ for reasons
                    // we obviously don't want it in user applications :D
                    priority::make_normal();
                    env::remove_var("DISPLAY");
                    env::set_var("WAYLAND_DISPLAY", &disp);
                    Ok(())
                }).spawn() {
                    warn!("Failed to spawn '{}': {:?}", prog, err);
                }
            },
            Err(err) => {
                warn!("Failed to recv: {:?}", err);
                thread::sleep(time::Duration::from_millis(1000));
            }
        }
    }
}
