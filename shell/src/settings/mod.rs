use gtk;
use gtk::prelude::*;
use relm::{Relm, Update, Widget, Component, ContainerWidget};
use relm_derive::Msg;
use relm_attributes::widget;
use protos::{gtkclient, outputs};

use self::Msg::*;

mod displays;
use self::displays::Displays;
// (seems like the view! macro recognizes relm components by lack of namespace)

#[derive(Clone)]
pub struct Model {
    dank_private: gtkclient::DankShellApi,
    shown: bool,
}

#[derive(Msg)]
pub enum Msg {
    Show,
    Hide,
    NextOutputState(outputs::OutputState),
}

#[widget]
impl Widget for Settings {
    fn model(dank_private: gtkclient::DankShellApi) -> Model {
        Model {
            dank_private,
            shown: false,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Show => { self.window.present(); },
            Hide => { self.window.hide(); },
            NextOutputState(s) => self.displays.emit(displays::Msg::NextOutputState(s)),
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            title: "Settings",
            gtk::Box {
                orientation: gtk::Orientation::Horizontal,
                #[name="stack_sidebar"]
                gtk::StackSidebar {
                },
                gtk::Separator {
                    orientation: gtk::Orientation::Vertical,
                },
                #[name="stack"]
                gtk::Stack {
                    hexpand: true,
                    #[name="displays"]
                    Displays(self.model.dank_private.clone()) {
                        child: {
                            title: "Displays",
                        },
                    },
                },
            },
        },
    }

    fn init_view(&mut self) {
        self.stack_sidebar.set_property_stack(Some(&self.stack));
    }
}
