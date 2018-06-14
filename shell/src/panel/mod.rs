use gtk;
use gtk::prelude::*;
use relm::{Relm, Update, Widget, Component, ContainerWidget};
use protos::gtkclient;

use self::Msg::*;

mod quick_launch;
mod clock;

#[derive(Debug, Serialize, Deserialize)]
pub enum WidgetConfig {
    QuickLaunch(quick_launch::LaunchConfig),
    Clock(clock::ClockConfig),
}

enum WidgetComponent {
    QuickLaunch(Component<quick_launch::Launch>),
    Clock(Component<clock::Clock>),
}

pub struct Model {
    layer_shell: gtkclient::LayerShellApi,
    dank_private: gtkclient::DankShellApi,
    widgets: Vec<WidgetConfig>,
}

#[derive(Msg)]
pub enum Msg {
    Reconfigure(Vec<WidgetConfig>),
}

pub struct Panel {
    model: Model,
    window: gtk::Window,
    hbox: gtk::Box,
    components: Vec<WidgetComponent>,
}

impl Update for Panel {
    type Model = Model;
    type ModelParam = (gtkclient::LayerShellApi, gtkclient::DankShellApi);
    type Msg = Msg;

    fn model(_relm: &Relm<Self>, (layer_shell, dank_private): (gtkclient::LayerShellApi, gtkclient::DankShellApi)) -> Model {
        Model {
            layer_shell,
            dank_private,
            widgets: vec![],
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Reconfigure(new_widgets) => {
                // Would be cool to diff and modify the config on existing components?
                // But for now just delete and recreate.
                for component in self.components.drain(0..) {
                    use self::WidgetComponent::*;
                    match component {
                        QuickLaunch(c) => self.hbox.remove_widget(c),
                        Clock(c) => self.hbox.remove_widget(c),
                    };
                }
                self.model.widgets = new_widgets;
                setup_widgets(&mut self.components, &self.hbox, &self.model);
            }
        }
    }
}

impl Widget for Panel {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(_relm: &Relm<Self>, mut model: Model) -> Self {
        let mut window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_title("Panel");
        window.set_default_size(640, 24);
        window.set_decorated(false);
        use gtkclient::lsr::{Anchor, RequestsTrait};
        let layer_surface = gtkclient::get_layer_surface(&mut model.layer_shell, &mut window, gtkclient::lsh::Layer::Top);
        layer_surface.set_anchor(Anchor::Bottom | Anchor::Left | Anchor::Right);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        window.add(&hbox);

        let mut components = Vec::new();
        setup_widgets(&mut components, &hbox, &model);

        window.show_all();
        Panel {
            model,
            window,
            hbox,
            components,
        }
    }
}

fn setup_widgets(components: &mut Vec<WidgetComponent>, hbox: &gtk::Box, model: &Model) {
    for widget in model.widgets.iter() {
        use self::WidgetConfig::*;
        let component = match widget {
            Clock(conf) => WidgetComponent::Clock(hbox.add_widget::<clock::Clock>(conf.clone())),
            QuickLaunch(conf) => WidgetComponent::QuickLaunch(hbox.add_widget::<quick_launch::Launch>((conf.clone(), model.dank_private.clone()))),
        };
        components.push(component);
    }
}
