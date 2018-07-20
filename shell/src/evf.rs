use parking_lot::RwLock;
use fragile::Sticky;
use relm::Component;
use crate::settings;

struct UnsafeSyncWrapper<T>(T);
unsafe impl<T> Sync for UnsafeSyncWrapper<T> {}

/// Some events arrive as soon as we get the globals.
/// But we configure our UI components later.
/// We need a forwarder to send events to them.
pub struct EventForwarder {
    settings: RwLock<Option<UnsafeSyncWrapper<Sticky<Component<settings::Settings>>>>>,
    settings_q: RwLock<Vec<settings::Msg>>,
}

impl EventForwarder {
    pub const fn new() -> EventForwarder {
        EventForwarder {
            settings: RwLock::new(None),
            settings_q: RwLock::new(Vec::new()),
        }
    }
}

static EVF: EventForwarder = EventForwarder::new();

pub fn set_settings(settings: Component<settings::Settings>) {
    let mut q = EVF.settings_q.write();
    let mut s = EVF.settings.write();
    for msg in q.drain(..) {
        settings.emit(msg);
    }
    *s = Some(UnsafeSyncWrapper(Sticky::new(settings)));
}

pub fn send_to_settings(msg: settings::Msg) {
    if let Some(ref s) = *EVF.settings.read() {
        s.0.get().emit(msg);
    } else {
        EVF.settings_q.write().push(msg);
    }
}
