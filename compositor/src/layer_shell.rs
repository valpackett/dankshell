use std::{ffi, mem};
use std::rc::Rc;
use std::cell::Cell;
use libc;
use weston_rs::{
    Compositor, Layer, View, Surface, ForeignType, libweston_sys,
    POSITION_BACKGROUND, POSITION_BOTTOM_UI, POSITION_UI, POSITION_LOCK,
};
use mut_static::MutStatic;
use wayland_sys::server::wl_resource;
use wayland_server::{NewResource, Resource, Display, LoopToken};
use wayland_server::commons::Implementation;
use protos::layer_shell::server::zxdg_layer_shell_v1 as lsh;
use protos::layer_shell::server::zxdg_layer_surface_v1 as lsr;
use authorization::{self, Permissions, LayerShellPermissions};
use surface_registry::{SURFACES, SurfaceListItem};
use COMPOSITOR;

struct Layers {
    background: Layer,
    bottom: Layer,
    top: Layer,
    overlay: Layer,
}

lazy_static! {
    static ref LAYERS: MutStatic<Layers> = MutStatic::new();
}

pub fn create_layers(compositor: &Compositor) {
    let mut layers = Layers {
        background: Layer::new(&compositor),
        bottom: Layer::new(&compositor),
        top: Layer::new(&compositor),
        overlay: Layer::new(&compositor),
    };
    layers.background.set_position(POSITION_BACKGROUND);
    layers.bottom.set_position(POSITION_BOTTOM_UI);
    layers.top.set_position(POSITION_UI);
    layers.overlay.set_position(POSITION_LOCK);
    LAYERS.set(layers).expect("layers MutStatic set");
}

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
    allowed: LayerShellPermissions,
    view: View,
    layer: lsh::Layer,
    anchor: lsr::Anchor,
    margin: Margin,
    req_size: (i32, i32),
    resource: Rc<Cell<Option<Resource<lsr::ZxdgLayerSurfaceV1>>>>,
}

impl LayerShellCtx {
    fn position(&self, surface_size: (i32, i32), output_size: (i32, i32)) -> (f32, f32) {
        let (w, h) = surface_size;
        let (output_w, output_h) = output_size;
        let (mut x, mut y) = (0.0, 0.0);
        if self.anchor.contains(lsr::Anchor::Top | lsr::Anchor::Bottom) ||
            !(self.anchor.contains(lsr::Anchor::Top) || self.anchor.contains(lsr::Anchor::Bottom)) {
            y = (output_h / 2 - h / 2) as f32;
        } else if self.anchor.contains(lsr::Anchor::Bottom) {
            y = (output_h - h - self.margin.bottom) as f32;
        } else if self.anchor.contains(lsr::Anchor::Top) {
            y = self.margin.top as f32;
        }
        if self.anchor.contains(lsr::Anchor::Left | lsr::Anchor::Right) ||
            !(self.anchor.contains(lsr::Anchor::Left) || self.anchor.contains(lsr::Anchor::Right)) {
            x = (output_w / 2 - w / 2) as f32;
        } else if self.anchor.contains(lsr::Anchor::Right) {
            x = (output_w - w - self.margin.right) as f32;
        } else if self.anchor.contains(lsr::Anchor::Left) {
            x = self.margin.left as f32;
        }
        (x, y)
    }

    fn next_size(&self, old_size: (i32, i32), output_size: (i32, i32)) -> (i32, i32) {
        let (output_w, output_h) = output_size;
        let (req_w, req_h) = self.req_size;
        let (mut w, mut h) = old_size;
        if req_w > 0 {
            w = req_w;
        }
        if req_h > 0 {
            h = req_h;
        }
        if self.anchor.contains(lsr::Anchor::Left | lsr::Anchor::Right) {
            w = output_w - self.margin.left - self.margin.right;
        }
        if self.anchor.contains(lsr::Anchor::Top | lsr::Anchor::Bottom) {
            h = output_h - self.margin.top - self.margin.bottom;
        }
        (w, h)
    }
}

struct LayerShellImpl {
}

unsafe impl Send for LayerShellImpl {}

impl Implementation<Resource<lsh::ZxdgLayerShellV1>, lsh::Request> for LayerShellImpl {
    fn receive(&mut self, msg: lsh::Request, resource: Resource<lsh::ZxdgLayerShellV1>) {
        let allowed = if let Some(Permissions { layer_shell, .. }) = authorization::resource_client_permissions(&resource) {
            info!("Permissions: {:?}", layer_shell);
            *layer_shell
        } else {
            warn!("No permissions found");
            return
        };
        let self::lsh::Request::GetLayerSurface { id, surface, layer, .. } = msg;
        // wayland-rs wraps user data, for unmanaged resources .get_user_data() returns a nullptr
        let surface_res_ptr = surface.c_ptr();
        let mut surface = get_weston_surface(surface_res_ptr);
        let _ = surface.set_role(ffi::CString::new("layer-shell").unwrap(), resource, 0);
        let view = View::new(&surface);
        let res_rc = Rc::new(Cell::new(None));
        surface.set_committed(|surface, sx, sy, mut ctx| {
            if !ctx.view.is_mapped() {
                use self::lsh::Layer::*;
                let mut layers = LAYERS.write().expect("layer MutStatic");
                match ctx.layer {
                    Background => {
                        if ctx.allowed.background {
                            layers.background.view_list_entry_insert(&mut ctx.view);
                        } else {
                            warn!("Background layer not allowed for this client");
                        }
                    },
                    Bottom => {
                        if ctx.allowed.bottom {
                            layers.bottom.view_list_entry_insert(&mut ctx.view)
                        } else {
                            warn!("Bottom layer not allowed for this client");
                        }
                    },
                    Top => {
                        if ctx.allowed.top {
                            layers.top.view_list_entry_insert(&mut ctx.view)
                        } else {
                            warn!("Top layer not allowed for this client");
                        }
                    },
                    Overlay => {
                        if ctx.allowed.overlay {
                            layers.overlay.view_list_entry_insert(&mut ctx.view)
                        } else {
                            warn!("Overlay layer not allowed for this client");
                        }
                    },
                }
                unsafe { (*ctx.view.as_ptr()).is_mapped = true };
            }
            // XXX: output is not assigned on first commit
            if let Some(output) = ctx.view.output() {
                let output_size = (output.width(), output.height());
                let (x, y) = ctx.position(surface.get_content_size(), output_size);
                ctx.view.set_position(x, y);
                let (old_w, old_h) = surface.get_content_size();
                let (new_w, new_h) = ctx.next_size((old_w, old_h), output_size);
                if new_w != old_w || new_h != old_h {
                    if let Some(res) = ctx.resource.take() {
                        use self::lsr::Event::Configure;
                        res.send(Configure { serial: 0, width: new_w as u32, height: new_h as u32 });
                        ctx.resource.set(Some(res));
                    }
                }
            }
            ctx.view.update_transform();
            surface.damage();
            surface.compositor_mut().schedule_repaint();
        }, LayerShellCtx {
            allowed,
            view,
            layer,
            anchor: lsr::Anchor::Top,
            margin: Margin::default(),
            req_size: (-1, -1),
            resource: res_rc.clone(),
        });
        res_rc.set(Some(id.implement(LayerSurfaceImpl { surface }, Some(|_, _| {}))));
        SURFACES.write().expect("surfaces write").push(SurfaceListItem::LayerShell(get_weston_surface(surface_res_ptr)));
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
            SetSize { width, height } => { ctx.req_size = (width as i32, height as i32); },
            SetAnchor { anchor } => { ctx.anchor = anchor },
            SetExclusiveZone { zone } => {},
            SetMargin { top, right, bottom, left } => { ctx.margin = Margin { top, right, bottom, left }; },
            SetKeyboardInteractivity { keyboard_interactivity } => {},
            GetPopup { popup } => {},
            AckConfigure { serial } => {},
            Destroy => {},
        }
        if let Some(output) = ctx.view.output() {
            let (old_w, old_h) = self.surface.get_content_size();
            let (new_w, new_h) = ctx.next_size((old_w, old_h), (output.width(), output.height()));
            if new_w != old_w || new_h != old_h {
                use self::lsr::Event::Configure;
                resource.send(Configure { serial: 0, width: new_w as u32, height: new_h as u32 });
            }
        }
    }
}
