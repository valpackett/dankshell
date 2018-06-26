use log::*;
use serde::{Serialize, Deserialize};
use gtk::prelude::*;
use gio::prelude::*;
use relm::{Widget, connect};
use relm_derive::Msg;
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
    icon_theme: gtk::IconTheme,
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
        use protos::gtkclient::lsr::{Anchor, RequestsTrait};
        let layer_surface = gtkclient::get_layer_surface(&mut self.model.layer_shell, &mut self.window, gtkclient::lsh::Layer::Top);
        layer_surface.set_margin(0, 0, 32, 0);
        layer_surface.set_size(320, 480);
        layer_surface.set_anchor(Anchor::Bottom | Anchor::Top | Anchor::Left);
        self.window.present();
        self.model.shown = true;
    }

    fn reload_apps(&mut self) {
        let (apps_time, apps) = elapsed::measure_time(|| gio::AppInfo::get_all());
        info!("Loading apps took {}", apps_time);
        for app in &apps {
            let row = gtk::Grid::new();
            row.set_column_spacing(10);
            row.set_row_spacing(10);
            // WHY CAN'T Image::new_from_*icon_* JUST TAKE FLAGS LIKE FORCE_SIZE?!?!? >_<
            let icon = gtk::Image::new_from_pixbuf(get_icon(&self.model.icon_theme, 32, app).as_ref());
            icon.set_margin_start(10);
            row.add(&icon);
            let label = gtk::Label::new(&app.get_display_name().unwrap_or("Unnamed App".to_owned()) as &str);
            label.set_margin_end(10);
            row.add(&label);
            row.show_all();
            self.app_list.add(&row);
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
            icon_theme: gtk::IconTheme::get_default().unwrap(),
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
                    vexpand: true,
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

fn get_icon(theme: &gtk::IconTheme, size: i32, app: &gio::AppInfo) -> Option<gdk_pixbuf::Pixbuf> {
    // TODO get display scale
    // For some reason GENERIC_FALLBACK spams "runtime check failed" and doesn't actually show a fallback??
    theme.lookup_by_gicon_for_scale(
        &app.get_icon()?, size, 1, gtk::IconLookupFlags::FORCE_SIZE
    )?.load_icon().ok()
}

/* TODO icon cache;

    let cache_key = app.get_icon().unwrap().to_string().unwrap_or("WTF".to_owned());

 * TODO app categories

    let mut cats = HashMap::new();
    let (cats_time, _) = elapsed::measure_time(|| {
        for app in &apps {
            let dapp = app.dynamic_cast::<DesktopAppInfo>().unwrap();
            if let Some(cats_s) = dapp.get_categories() {
                for (idx, cat) in cats_s.split(';').enumerate() {
                    let cat = cat.trim();
                    if !cats.contains_key::<str>(&cat) {
                        cats.insert(cat.to_owned(), vec![idx]);
                    } else {
                        cats.get_mut::<str>(&cat).unwrap().push(idx);
                    }
                }
            }
        }
    });
    info!("Categorizing apps took {}", cats_time);

*/
