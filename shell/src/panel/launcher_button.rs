use std::rc::Rc;
use serde::{Serialize, Deserialize};
use gtk;
use gtk::ButtonExt;
use relm::{Widget, Component, connect};
use relm_derive::Msg;
use relm_attributes::widget;
use crate::launcher;

use self::Msg::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherButtonConfig {
    label: String,
    icon: String,
}

impl Default for LauncherButtonConfig {
    fn default() -> LauncherButtonConfig {
        LauncherButtonConfig {
            label: "".to_owned(),
            icon: "applications-other".to_owned(),
        }
    }
}

pub struct Model {
    config: LauncherButtonConfig,
    launcher: Rc<Component<launcher::Launcher>>,
}

#[derive(Msg)]
pub enum Msg {
    Click,
}

#[widget]
impl Widget for LauncherButton {
    fn model((config, launcher): (LauncherButtonConfig, Rc<Component<launcher::Launcher>>)) -> Model {
        Model {
            config,
            launcher,
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Click => {
                self.model.launcher.emit(launcher::Msg::ToggleVisibility);
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
