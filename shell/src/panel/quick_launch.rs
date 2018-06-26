use serde::{Serialize, Deserialize};
use gtk;
use gtk::ButtonExt;
use relm::{Widget, connect};
use relm_derive::Msg;
use relm_attributes::widget;
use protos::gtkclient;

use self::Msg::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    label: String,
    icon: String,
    cmd: String,
}

impl Default for LaunchConfig {
    fn default() -> LaunchConfig {
        LaunchConfig {
            label: "".to_owned(),
            icon: "utilities-terminal".to_owned(),
            cmd: "gnome-terminal".to_owned(),
        }
    }
}

pub struct Model {
    config: LaunchConfig,
    dank_private: gtkclient::DankShellApi,
}

#[derive(Msg)]
pub enum Msg {
    Click,
}

#[widget]
impl Widget for Launch {
    fn model((config, dank_private): (LaunchConfig, gtkclient::DankShellApi)) -> Model {
        Model {
            config,
            dank_private,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Click => {
                use protos::gtkclient::api::RequestsTrait;
                self.model.dank_private.spawn_program(self.model.config.cmd.clone(), None);
            },
        }
    }

    view! {
        gtk::Button {
            label: &self.model.config.label,
            image: &gtk::Image::new_from_icon_name(Some(&self.model.config.icon as &str), gtk::IconSize::LargeToolbar.into()),
            always_show_image: true,
            clicked => Click,
        },
    }
}
