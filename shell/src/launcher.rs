use gtk;
use gtk::prelude::*;
use relm::Widget;
use relm_attributes::widget;
use protos::gtkclient;
use desktop_entries::ENTRIES;

use self::Msg::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
}

impl Default for LauncherConfig {
    fn default() -> LauncherConfig {
        LauncherConfig {
        }
    }
}

#[derive(Clone)]
pub struct Model {
    layer_shell: gtkclient::LayerShellApi,
    dank_private: gtkclient::DankShellApi,
    config: LauncherConfig,
    shown: bool,
}

#[derive(Msg)]
pub enum Msg {
    Reconfigure(LauncherConfig),
    Show,
    Hide,
    ToggleVisibility,
    ReloadApps,
    RunEntry(usize),
}

impl Launcher {
    fn hide(&mut self) {
        if !self.model.shown {
            return
        }
        self.window.hide(); // Destroys the surface
        self.model.shown = false;
    }

    fn show(&mut self) {
        if self.model.shown {
            return
        }
        // So we create the layer-surface here
        use gtkclient::lsr::{Anchor, RequestsTrait};
        let layer_surface = gtkclient::get_layer_surface(&mut self.model.layer_shell, &mut self.window, gtkclient::lsh::Layer::Top);
        layer_surface.set_margin(0, 0, 32, 0);
        layer_surface.set_size(320, 480);
        layer_surface.set_anchor(Anchor::Bottom | Anchor::Top | Anchor::Left);
        self.window.present();
        self.model.shown = true;
    }

    fn reload_apps(&mut self) {
        let entries = ENTRIES.read();
        for app in entries.apps.iter() {
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            let label = gtk::Label::new(&app.name as &str);
            hbox.pack_start(&label, true, true, 10);
            hbox.show_all();
            self.app_list.add(&hbox);
        }
    }
}

#[widget]
impl Widget for Launcher {
    fn model((layer_shell, dank_private): (gtkclient::LayerShellApi, gtkclient::DankShellApi)) -> Model {
        Model {
            layer_shell,
            dank_private,
            config: Default::default(),
            shown: false,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Reconfigure(new_config) => {
                self.model.config = new_config;
            },
            Show => { self.show(); },
            Hide => { self.hide(); },
            ToggleVisibility => {
                if self.model.shown {
                    self.hide();
                } else {
                    self.show();
                }
            },
            ReloadApps => { self.reload_apps(); },
            RunEntry(idx) => {
                warn!("TODO / run {:?}", idx);
                self.hide();
            },
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            title: "Launcher",
            decorated: false,
            visible: false,
            gtk::Box {
                orientation: gtk::Orientation::Vertical,
                gtk::ScrolledWindow {
                    child: { expand: true },
                    #[name="app_list"]
                    gtk::ListBox {
                        activate_on_single_click: true,
                        row_activated(_, row) => RunEntry(row.get_index() as usize),
                    },
                },
            },
        },
    }
}
