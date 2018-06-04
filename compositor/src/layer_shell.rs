use std::{ffi, mem};
use libc;
use weston_rs::{View, Surface, ForeignType, libweston_sys};
use wayland_sys::server::wl_resource;
use wayland_server::{NewResource, Resource, Display, LoopToken};
use wayland_server::commons::Implementation;
use protos::layer_shell::server::zxdg_layer_shell_v1 as lsh;
use protos::layer_shell::server::zxdg_layer_surface_v1 as lsr;
use {COMPOSITOR, TOP_LAYER};

extern "C" {
    fn wl_resource_get_user_data(res: *mut wl_resource) -> *mut libc::c_void;
}

// The LayerSurfaceImpl needs to own the object, so we don't use the SurfaceRef,
// just a leaked (manually dropped) Surface.
//
// Shouldn't be a problem, because the Impl methods are called to handle requests
// from that client with that surface, so when the surface gets dropped, the impl
// shouldn't get any requests anymore.
fn get_weston_surface(res: *mut wl_resource) -> mem::ManuallyDrop<Surface> {
    let ptr = unsafe { wl_resource_get_user_data(res) } as *mut libweston_sys::weston_surface;
    debug!("user data (libweston surface) ptr {:?}", ptr);
    mem::ManuallyDrop::new(unsafe { Surface::from_ptr(ptr) })
}

#[derive(Default)]
struct Margin {
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
}

/// The data for each surface, kept in its committed_private (user data) field.
struct LayerShellCtx {
    view: View,
    anchor: lsr::Anchor,
    margin: Margin,
}

struct LayerShellImpl {
}

unsafe impl Send for LayerShellImpl {}

impl Implementation<Resource<lsh::ZxdgLayerShellV1>, lsh::Request> for LayerShellImpl {
    fn receive(&mut self, msg: lsh::Request, resource: Resource<lsh::ZxdgLayerShellV1>) {
        let self::lsh::Request::GetLayerSurface { id, surface, .. } = msg;
        // wayland-rs wraps user data, for unmanaged resources .get_user_data() returns a nullptr
        let mut surface = get_weston_surface(surface.c_ptr());
        let _ = surface.set_role(ffi::CString::new("layer-shell").unwrap(), resource, 0);
        let view = View::new(&surface);
        surface.set_committed(|surface, sx, sy, mut ctx| {
            if !ctx.view.is_mapped() {
                let mut top_layer = TOP_LAYER.write().expect("top_layer MutStatic");
                top_layer.view_list_entry_insert(&mut ctx.view);
                unsafe { (*ctx.view.as_ptr()).is_mapped = true };
            }
            let (w, h) = surface.get_content_size();
            let (mut x, mut y) = (0.0, 0.0);
            // XXX: output is not assigned on first commit
            if let Some(output) = ctx.view.output() {
                if ctx.anchor.contains(lsr::Anchor::Top | lsr::Anchor::Bottom) ||
                    !(ctx.anchor.contains(lsr::Anchor::Top) || ctx.anchor.contains(lsr::Anchor::Bottom)) {
                    y = (output.height() / 2 - h / 2) as f32;
                } else if ctx.anchor.contains(lsr::Anchor::Bottom) {
                    y = (output.height() - h - ctx.margin.bottom) as f32;
                } else if ctx.anchor.contains(lsr::Anchor::Top) {
                    y = ctx.margin.top as f32;
                }
                if ctx.anchor.contains(lsr::Anchor::Left | lsr::Anchor::Right) ||
                    !(ctx.anchor.contains(lsr::Anchor::Left) || ctx.anchor.contains(lsr::Anchor::Right)) {
                    x = (output.width() / 2 - w / 2) as f32;
                } else if ctx.anchor.contains(lsr::Anchor::Right) {
                    x = (output.width() - w - ctx.margin.right) as f32;
                } else if ctx.anchor.contains(lsr::Anchor::Left) {
                    x = ctx.margin.left as f32;
                }
            }
            ctx.view.set_position(x, y);
            ctx.view.update_transform();
            surface.damage();
            surface.compositor_mut().schedule_repaint();
        }, LayerShellCtx {
            view,
            anchor: lsr::Anchor::Top,
            margin: Margin::default(),
        });
        id.implement(LayerSurfaceImpl { surface }, Some(|_, _| {}));
    }
}

pub fn register_layer_shell(display: &mut Display, token: LoopToken) {
    display.create_global::<lsh::ZxdgLayerShellV1, _>(&token, 1, |_, res: NewResource<lsh::ZxdgLayerShellV1>| {
        res.implement(LayerShellImpl { }, Some(|_, _| {}));
    });
}

struct LayerSurfaceImpl {
    surface: mem::ManuallyDrop<Surface>,
}

unsafe impl Send for LayerSurfaceImpl {}

impl Implementation<Resource<lsr::ZxdgLayerSurfaceV1>, lsr::Request> for LayerSurfaceImpl {
    fn receive(&mut self, msg: lsr::Request, resource: Resource<lsr::ZxdgLayerSurfaceV1>) {
        let ctx : &mut LayerShellCtx = unsafe { self.surface.committed_private_mut() };
        use self::lsr::Request::*;
        match msg {
            SetSize { width, height } => {},
            SetAnchor { anchor } => { ctx.anchor = anchor },
            SetExclusiveZone { zone } => {},
            SetMargin { top, right, bottom, left } => { ctx.margin = Margin { top, right, bottom, left } },
            SetKeyboardInteractivity { keyboard_interactivity } => {},
            GetPopup { popup } => {},
            AckConfigure { serial } => {},
            Destroy => {},
        }
    }
}
