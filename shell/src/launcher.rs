use gtk;
use gtk::prelude::*;
use relm::Widget;
use relm_attributes::widget;
use protos::gtkclient;

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
            }
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            title: "Launcher",
            decorated: false,
            visible: false,
            gtk::Button {
                label: "TODO",
            },
        },
    }
}
