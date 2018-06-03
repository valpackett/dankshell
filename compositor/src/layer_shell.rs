use std::ffi;
use libc;
use weston_rs::{View, SurfaceRef, ForeignTypeRef, libweston_sys};
use wayland_sys::server::wl_resource;
use wayland_server::{NewResource, Resource, Display, LoopToken};
use wayland_server::commons::Implementation;
use protos::layer_shell::server::zxdg_layer_shell_v1 as lsh;
use {COMPOSITOR, TOP_LAYER};

extern "C" {
    fn wl_resource_get_user_data(res: *mut wl_resource) -> *mut libc::c_void;
}

struct LayerShellImpl {
}

unsafe impl Send for LayerShellImpl {}

impl Implementation<Resource<lsh::ZxdgLayerShellV1>, lsh::Request> for LayerShellImpl {
    fn receive(&mut self, msg: lsh::Request, resource: Resource<lsh::ZxdgLayerShellV1>) {
        match msg {
            lsh::Request::GetLayerSurface { surface, .. } => {
                // wayland-rs wraps user data, for unmanaged resources .get_user_data() returns a nullptr
                let ptr = unsafe { wl_resource_get_user_data(surface.c_ptr()) } as *mut libweston_sys::weston_surface;
                debug!("get_layer_surface: user data ptr {:?}", ptr);
                let surface = unsafe { SurfaceRef::from_ptr_mut(ptr) };
                let _ = surface.set_role(ffi::CString::new("layer-shell").unwrap(), resource, 0);
                let view = View::new(&surface);
                surface.set_committed(|surface, sx, sy, mut view| {
                    if view.is_mapped() {
                        return
                    }
                    let mut compositor = COMPOSITOR.write().expect("compositor MutStatic");
                    let mut top_layer = TOP_LAYER.write().expect("top_layer MutStatic");
                    unsafe { (*view.as_ptr()).is_mapped = true };
                    top_layer.view_list_entry_insert(&mut view);
                    view.set_position(0.0, 0.0);
                    view.update_transform();
                    surface.damage();
                    compositor.schedule_repaint();
                }, view);
            }
        }
    }
}

pub fn register_layer_shell(display: &mut Display, token: LoopToken) {
    display.create_global::<lsh::ZxdgLayerShellV1, _>(&token, 1, |_, res: NewResource<lsh::ZxdgLayerShellV1>| {
        res.implement(LayerShellImpl { }, Some(|_, _| {}));
    });
}
