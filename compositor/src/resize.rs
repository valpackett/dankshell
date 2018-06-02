use std::cmp;
use weston_rs::*;
use ctx::SurfaceContext;

/// Grab handler for resizing windows
pub struct ResizeGrab<'a> {
    pub dsurf: &'a mut DesktopSurfaceRef<SurfaceContext>,
    pub edges: Resize,
    pub width: i32,
    pub height: i32,
}

impl<'a> PointerGrab for ResizeGrab<'a> {
    fn motion(&mut self, pointer: &mut PointerRef, _time: &libc::timespec, event: PointerMotionEvent) {
        pointer.moove(event);
        let sctx = self.dsurf.borrow_user_data().expect("user_data");
        let (from_x, from_y) = sctx.view.from_global_fixed(pointer.grab_x(), pointer.grab_y());
        let (to_x, to_y) = sctx.view.from_global_fixed(pointer.x(), pointer.y());
        let mut width = self.width;
        if self.edges.contains(Resize::Left) {
            width += wl_fixed_to_int(from_x - to_x);
        } else if self.edges.contains(Resize::Right) {
            width += wl_fixed_to_int(to_x - from_x);
        }
        let mut height = self.height;
        if self.edges.contains(Resize::Top) {
            height += wl_fixed_to_int(from_y - to_y);
        } else if self.edges.contains(Resize::Bottom) {
            height += wl_fixed_to_int(to_y - from_y);
        }
        let mut min_size = self.dsurf.get_min_size();
        min_size.width = cmp::max(1, min_size.width);
        min_size.height = cmp::max(1, min_size.height);
        let max_size = self.dsurf.get_max_size();
        if width < min_size.width {
            width = min_size.width;
        } else if max_size.width > 0 && width > max_size.width {
            width = max_size.width;
        }
        if height < min_size.height {
            height = min_size.height;
        } else if max_size.width > 0 && width > max_size.width {
            // is it right that we're doing the width thing again, not height? (copied from weston desktop shell)
            width = max_size.width;
        }
        self.dsurf.set_size(width, height);
    }

    fn button(&mut self, pointer: &mut PointerRef, _time: &libc::timespec, _button: u32, state: ButtonState) {
        if pointer.button_count() == 0 && state == ButtonState::Released {
            self.cancel(pointer);
        }
    }

    fn cancel(&mut self, pointer: &mut PointerRef) {
        let sctx = self.dsurf.borrow_user_data().expect("user_data");
        self.dsurf.set_resizing(false);
        sctx.resize_edges = Resize::None;
        pointer.end_grab();
    }
}

// TODO: TouchGrab
