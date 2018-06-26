use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};
use gtk;
use gtk::LabelExt;
use relm::{Relm, Widget, interval};
use relm_derive::Msg;
use relm_attributes::widget;

use self::Msg::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockConfig {
    format: String,
}

impl Default for ClockConfig {
    fn default() -> ClockConfig {
        ClockConfig {
            format: "%H:%M:%S".to_owned(),
        }
    }
}

pub struct Model {
    time: DateTime<Local>,
    config: ClockConfig,
}

#[derive(Msg)]
pub enum Msg {
    Tick,
}

#[widget]
impl Widget for Clock {
    fn model(config: ClockConfig) -> Model {
        Model {
            time: Local::now(),
            config,
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        interval(relm.stream(), 1000, || Tick);
    }

    fn update(&mut self, event: Msg) {
        match event {
            Tick => self.model.time = Local::now(),
        }
    }

    view! {
        gtk::Label {
            text: &self.model.time.format(&self.model.config.format).to_string(),
        },
    }
}
