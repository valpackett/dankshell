use weston_rs::*;
use crate::ctx::SurfaceContext;

/// Grab handler for moving windows
pub struct MoveGrab<'a> {
    pub dsurf: &'a mut DesktopSurfaceRef<SurfaceContext>,
    pub dx: f64,
    pub dy: f64,
}

impl<'a> PointerGrab for MoveGrab<'a> {
    fn motion(&mut self, pointer: &mut PointerRef, _time: &libc::timespec, event: PointerMotionEvent) {
        pointer.moove(event);
        let sctx = self.dsurf.borrow_user_data().expect("user_data");
        sctx.view.set_position((wl_fixed_to_double(pointer.x()) + self.dx) as f32, (wl_fixed_to_double(pointer.y()) + self.dy) as f32);
        self.dsurf.surface().compositor_mut().schedule_repaint();
    }

    fn button(&mut self, pointer: &mut PointerRef, _time: &libc::timespec, _button: u32, state: ButtonState) {
        if pointer.button_count() == 0 && state == ButtonState::Released {
            pointer.end_grab();
        }
    }

    fn cancel(&mut self, pointer: &mut PointerRef) {
        pointer.end_grab();
    }
}

// TODO: TouchGrab
