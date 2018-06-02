use std::cell;
use weston_rs::*;
use ctx::SurfaceContext;
use desktop::DesktopImpl;
use DESKTOP;

fn activate(view: &mut ViewRef, seat: &SeatRef, flags: ActivateFlag) {
    let main_surf = unsafe { SurfaceRef::from_ptr(view.surface().main_surface().as_ptr()) };
    if let Some(dsurf) = DesktopSurfaceRef::<SurfaceContext>::from_surface(&main_surf) {
        let mut desktop = DESKTOP.write().expect("desktop MutStatic");
        let desktop_impl = desktop.api().as_any().downcast_mut::<DesktopImpl>().expect("DesktopImpl downcast");

        view.activate(&seat, flags);

        // Re-insert into the layer to put on top visually
        if view.layer_link().layer.is_null() {
            // Except for newly created surfaces (?)
            // e.g. w/o this, clicking a GTK menu action that spawns a new window would freeze
            return
        }
        view.geometry_dirty();
        view.layer_entry_remove();
        desktop_impl.windows_layer.view_list_entry_insert(view);
        dsurf.propagate_layer();
        view.geometry_dirty();
        dsurf.surface_mut().damage();
    }
}

pub fn click_activate(p: &mut PointerRef) {
    if !p.is_default_grab() {
        return;
    }
    if let Some(focus_view) = p.focus_mut() {
        activate(focus_view, p.seat(), ActivateFlag::CONFIGURE | ActivateFlag::CLICKED);
    }
}


pub fn keyboard_focus_listener() -> mem::ManuallyDrop<Box<WlListener<KeyboardRef>>> {
    let focused_surface = cell::RefCell::new(None); // in desktop-shell this is part of seat state
    WlListener::new(Box::new(move |p: &mut KeyboardRef| {
        if let Some(old_focus) = focused_surface.replace(p.focus().map(|f| unsafe { SurfaceRef::from_ptr(f.as_ptr()) })) {
            if let Some(dsurf) = DesktopSurfaceRef::<SurfaceContext>::from_surface(&old_focus) {
                if let Some(sctx) = dsurf.borrow_user_data() {
                    sctx.focus_count -= 1;
                    if sctx.focus_count == 0 {
                        dsurf.set_activated(false);
                    }
                }
            }
        }

        if let Some(focus) = *focused_surface.borrow() {
            if let Some(dsurf) = DesktopSurfaceRef::<SurfaceContext>::from_surface(&focus) {
                if let Some(sctx) = dsurf.borrow_user_data() {
                    if sctx.focus_count == 0 {
                        dsurf.set_activated(true);
                    }
                    sctx.focus_count += 1;
                }
            }
        }
    }))
}
