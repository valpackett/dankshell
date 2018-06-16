use std::{env, ffi, mem, ptr};
use weston_rs::*;

pub enum SelectedBackend {
    Drm(DrmOutputImpl),
    Windowed(WindowedOutputImpl),
}

pub fn start_backend(compositor: &mut CompositorRef, event_loop: &mut EventLoop) -> SelectedBackend {
    let is_under_wayland = env::var("WAYLAND_DISPLAY").is_ok() || env::var("WAYLAND_SOCKET").is_ok();
    let is_under_loginw = env::var("LOGINW_FD").is_ok();
    if is_under_wayland { // TODO X11
        let _backend = WaylandBackend::new(&compositor, WaylandBackendConfigBuilder::default().build().unwrap());
        let output_api = unsafe { WindowedOutputImpl::from_ptr(compositor.get_windowed_output().expect("get_windowed_output").as_ptr()) };
        output_api.create_head(&compositor, "weston-rs simple example");
        SelectedBackend::Windowed(output_api)
    } else {
        if is_under_loginw {
            let launcher = LoginwLauncher::connect(&compositor, event_loop, 0, &ffi::CString::new("default").unwrap(), false).expect("connect");
            compositor.set_launcher(launcher);
        } else {
            #[cfg(target_os = "linux")]
            unsafe {
                let mut wrapper: *mut libweston_sys::weston_launcher = ptr::null_mut();
                //wrapper.iface = &libweston_sys::launcher_logind_iface as *const _ as *mut _;
                let seatname = ffi::CString::new("seat0").unwrap();
                libweston_sys::launcher_logind_iface.connect.unwrap()(&mut wrapper as *mut _, compositor.as_ptr(), 0, seatname.as_bytes_with_nul() as *const _ as *const u8, false);
                compositor.set_launcher_raw(wrapper);
            }
        }
        let _backend = DrmBackend::new(&compositor, DrmBackendConfigBuilder::default().build().unwrap());
        let output_api = unsafe { DrmOutputImpl::from_ptr(compositor.get_drm_output().expect("get_drm_output").as_ptr()) };
        SelectedBackend::Drm(output_api)
    }
}

pub fn heads_changed_listener(be: SelectedBackend) -> mem::ManuallyDrop<Box<WlListener<CompositorRef>>> {
    WlListener::new(Box::new(move |compositor: &mut CompositorRef| {
        let compositor_ = unsafe { CompositorRef::from_ptr_mut(compositor.as_ptr()) };
        for head in compositor.iterate_heads() {
            if head.is_connected() && !head.is_enabled() {
                head_enable(compositor_, head, &be);
            } else if !head.is_connected() && head.is_enabled() {
                drop(head.output_owned());
            } else if head.is_enabled() && head.is_device_changed() {
                warn!("Detected monitor change on head '{:?}'\n", head.get_name());
            }
            head.reset_device_changed();
        }
    }))
}

fn head_enable(compositor: &mut CompositorRef, head: &mut HeadRef, be: &SelectedBackend) {
    if head.output().is_some() {
        return
    }
    if let Some(mut output) = compositor.create_output_with_head(head) {
        match be {
            SelectedBackend::Drm(output_api) => {
                output_api.set_mode(&output, DrmBackendOutputMode::Current, None);
                output.set_scale(1);
                output.set_extra_scale(1.0);
                output.set_transform(0);
                output_api.set_gbm_format(&output, None);
            },
            SelectedBackend::Windowed(output_api) => {
                output.set_scale(1);
                output.set_extra_scale(1.0);
                output.set_transform(0);
                output_api.output_set_size(&output, 1280, 720);
            },
        }
        if !output.enable() {
            error!("Could not enable output for head {:?}\n", head.get_name());
        } else {
            let _ = mem::ManuallyDrop::new(output); // keep (do not destroy)
        }
    } else {
        error!("Could not create an output for head {:?}\n", head.get_name());
    }
}

// TODO: output destruction on last head destruction (`wet_head_tracker_create`)
