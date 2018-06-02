use std::{env, ffi, mem};
use weston_rs::*;

pub enum SelectedBackend {
    Drm(DrmOutputImpl),
    Windowed(WindowedOutputImpl),
}

pub fn start_backend(compositor: &mut CompositorRef, event_loop: &mut EventLoop) -> SelectedBackend {
    if env::var("LOGINW_FD").is_ok() {
        let launcher = LoginwLauncher::connect(&compositor, event_loop, 0, &ffi::CString::new("default").unwrap(), false).expect("connect");
        compositor.set_launcher(launcher);
        let _backend = DrmBackend::new(&compositor, DrmBackendConfigBuilder::default().build().unwrap());
        let output_api = unsafe { DrmOutputImpl::from_ptr(compositor.get_drm_output().expect("get_drm_output").as_ptr()) };
        SelectedBackend::Drm(output_api)
    } else {
        let _backend = WaylandBackend::new(&compositor, WaylandBackendConfigBuilder::default().build().unwrap());
        let output_api = unsafe { WindowedOutputImpl::from_ptr(compositor.get_windowed_output().expect("get_windowed_output").as_ptr()) };
        output_api.create_head(&compositor, "weston-rs simple example");
        SelectedBackend::Windowed(output_api)
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
                eprintln!("Detected monitor change on head '{:?}'\n", head.get_name());
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
            eprintln!("Could not enable output for head {:?}\n", head.get_name());
        } else {
            let _ = mem::ManuallyDrop::new(output); // keep (do not destroy)
        }
    } else {
        eprintln!("Could not create an output for head {:?}\n", head.get_name());
    }
}

// TODO: output destruction on last head destruction (`wet_head_tracker_create`)
