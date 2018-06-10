use std::any;
use weston_rs::*;
use ctx::SurfaceContext;
use moove::MoveGrab;
use resize::ResizeGrab;
use surface_registry::{SURFACES, SurfaceListItem};
use COMPOSITOR;

/// Data for the Desktop API implementation
pub struct DesktopImpl {
    pub windows_layer: Layer,
}

impl DesktopImpl {
    pub fn new(compositor: &Compositor) -> DesktopImpl {
        let mut windows_layer = Layer::new(&compositor);
        windows_layer.set_position(POSITION_NORMAL);

        DesktopImpl {
            windows_layer,
        }
    }
}

impl DesktopApi<SurfaceContext> for DesktopImpl {
    fn as_any(&mut self) -> &mut any::Any { self }

    fn surface_added(&mut self, dsurf: &mut DesktopSurfaceRef<SurfaceContext>) {
        let mut view = dsurf.create_view();
        self.windows_layer.view_list_entry_insert(&mut view);
        let mut compositor = COMPOSITOR.write().expect("compositor MutStatic");
        dsurf.surface_mut().damage();
        compositor.schedule_repaint();
        dsurf.set_activated(true);
        view.activate(&compositor.first_seat().expect("first_seat"), ActivateFlag::CONFIGURE);
        let _ = dsurf.set_user_data(Box::new(SurfaceContext {
            view,
            resize_edges: Resize::None,
            last_width: 0.0,
            last_height: 0.0,
            focus_count: 1,
        }));
        SURFACES.write().expect("surfaces write").push(SurfaceListItem::Desktop(dsurf.as_ptr()))
    }

    fn surface_removed(&mut self, dsurf: &mut DesktopSurfaceRef<SurfaceContext>) {
        let mut sctx = dsurf.get_user_data().expect("user_data");
        dsurf.unlink_view(&mut sctx.view);
        SURFACES.write().expect("surfaces write").remove_item(&SurfaceListItem::Desktop(dsurf.as_ptr()));
        // sctx dropped here, destroying the view
    }

    fn committed(&mut self, dsurf: &mut DesktopSurfaceRef<SurfaceContext>, _sx: i32, _sy: i32) {
        let sctx = dsurf.borrow_user_data().expect("user_data");
        let surface = dsurf.surface();
        let (from_x, from_y) = sctx.view.from_global_float(0.0, 0.0);
        let (to_x, to_y) = sctx.view.from_global_float(
            if sctx.resize_edges.contains(Resize::Left) { sctx.last_width - surface.width() as f32 } else { 0.0 },
            if sctx.resize_edges.contains(Resize::Top) { sctx.last_height - surface.height() as f32 } else { 0.0 },
        );
        let (orig_x, orig_y) = sctx.view.get_position();
        sctx.view.set_position(orig_x + to_x - from_x, orig_y + to_y - from_y);
        sctx.last_width = surface.width() as f32;
        sctx.last_height = surface.height() as f32;
    }

    fn moove(&mut self, dsurf: &mut DesktopSurfaceRef<SurfaceContext>, seat: &mut SeatRef, serial: u32) {
        let sctx = dsurf.borrow_user_data().expect("user_data");
        if let Some(pointer) = seat.pointer_mut() {
            if let Some(focus) = pointer.focus() {
                if pointer.button_count() > 0 && serial == pointer.grab_serial() &&
                    focus.surface().main_surface().as_ptr() == dsurf.surface().as_ptr() {
                    let (view_x, view_y) = sctx.view.get_position();
                    let grab = MoveGrab {
                        dsurf: unsafe { DesktopSurfaceRef::from_ptr_mut(dsurf.as_ptr()) },
                        dx: f64::from(view_x) - wl_fixed_to_double(pointer.grab_x()),
                        dy: f64::from(view_y) - wl_fixed_to_double(pointer.grab_y()),
                    };
                    pointer.start_grab(grab);
                }
            }
        }
    }

    fn resize(&mut self, dsurf: &mut DesktopSurfaceRef<SurfaceContext>, seat: &mut SeatRef, serial: u32, edges: Resize) {
        if edges == Resize::None || edges.contains(Resize::Left | Resize::Right) || edges.contains(Resize::Top | Resize::Bottom) {
            return
        }
        let sctx = dsurf.borrow_user_data().expect("user_data");
        if let Some(pointer) = seat.pointer_mut() {
            if let Some(focus) = pointer.focus() {
                if pointer.button_count() > 0 && serial == pointer.grab_serial() &&
                    focus.surface().main_surface().as_ptr() == dsurf.surface().as_ptr() {
                    let geom = dsurf.get_geometry();
                    let grab = ResizeGrab {
                        dsurf: unsafe { DesktopSurfaceRef::from_ptr_mut(dsurf.as_ptr()) },
                        edges,
                        width: geom.width,
                        height: geom.height,
                    };
                    dsurf.set_resizing(true);
                    sctx.resize_edges = edges;
                    pointer.start_grab(grab);
                }
            }
        }
    }
}
