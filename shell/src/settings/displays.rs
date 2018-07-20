use gtk;
use gtk::prelude::*;
use relm::{Relm, Update, Widget, Component, ContainerWidget};
use relm_derive::Msg;
use relm_attributes::widget;
use protos::{gtkclient, outputs};

use self::Msg::*;

#[derive(Clone)]
pub struct Model {
    dank_private: gtkclient::DankShellApi,
}

#[derive(Msg)]
pub enum Msg {
    NextOutputState(outputs::OutputState),
}

#[widget]
impl Widget for Displays {
    fn model(dank_private: gtkclient::DankShellApi) -> Model {
        Model {
            dank_private,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            NextOutputState(s) => println!("{:?}", s),
        }
    }

    view! {
        gtk::Box {
            orientation: gtk::Orientation::Vertical,
        },
    }
}
