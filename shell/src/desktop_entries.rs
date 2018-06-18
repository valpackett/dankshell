use std::{fs, sync, thread};
use std::io::Read;
use std::path::Path;
use std::boxed::FnBox;
use std::collections::HashMap;
use parking_lot::RwLock;
use ini::Ini;
use glob::glob;

trait GetterExt {
    fn get_bool(&self, key: &str) -> bool;
    fn get_list(&self, key: &str) -> Vec<String>;
}

impl GetterExt for HashMap<String, String> {
    fn get_bool(&self, key: &str) -> bool {
        self.get(key).map(|x| x == "true").unwrap_or(false)
    }

    fn get_list(&self, key: &str) -> Vec<String> {
        self.get(key).map(|x| x.split(";").map(|y| y.to_owned()).collect()).unwrap_or_else(|| Vec::new())
    }
}

pub static ENTRIES: RwLock<DesktopEntries> = RwLock::new(DesktopEntries::new());

#[derive(Debug)]
pub struct DesktopEntries {
    pub apps: Vec<Application>,
    /// Categories stored as indices into the apps vector
    pub cats: Option<HashMap<String, Vec<usize>>>,
}

impl DesktopEntries {
    const fn new() -> DesktopEntries {
        DesktopEntries {
            apps: Vec::new(),
            cats: None,
        }
    }
}

#[derive(Debug)]
pub struct Application {
    pub name: String,
    pub icon: Option<String>,
    pub exec: String,
    pub categories: Vec<String>,
    pub terminal: bool,
    pub startup_notify: bool,
}

fn read_entry(mut read: impl Read) -> Option<Application> {
    use self::GetterExt;
    let ini = Ini::read_from(&mut read).ok()?;
    let desk = ini.section(Some("Desktop Entry"))?;
    Some(Application {
        name: desk.get("Name")?.to_owned(),
        icon: desk.get("Icon").map(|x| x.to_owned()),
        exec: desk.get("Exec")?.to_owned(),
        categories: desk.get_list("Categories"),
        terminal: desk.get_bool("Name"),
        startup_notify: desk.get_bool("StartupNotify"),
    })
}

fn read_entries<P: AsRef<Path>>(files: impl Iterator<Item = P>) -> DesktopEntries {
    let mut apps = Vec::new();
    let mut cats = HashMap::new();
    for path in files {
        match fs::File::open(path.as_ref()) {
            Ok(file) => {
                if let Some(entry) = read_entry(file) {
                    let idx = apps.len(); // before push
                    for cat in entry.categories.iter() {
                        if !cats.contains_key::<str>(&cat) {
                            cats.insert(cat.clone(), vec![idx]);
                        } else {
                            cats.get_mut::<str>(&cat).unwrap().push(idx);
                        }
                    }
                    apps.push(entry);
                } else {
                    warn!("Could not parse file '{:?}'", path.as_ref())
                }
            },
            Err(e) => warn!("Could not open file '{:?}': {:?}", path.as_ref(), e),
        }
    }
    DesktopEntries {
        apps,
        cats: Some(cats),
    }
}

fn update_entries() {
    let globs = glob("/usr/local/share/applications/*.desktop").unwrap().filter_map(|res| res.ok());
    let new_entries = read_entries(globs);
    warn!("entries: {:?}", new_entries);
    let mut entries = ENTRIES.write();
    *entries = new_entries;
}

pub type UpdateCallback = Box<FnBox() + Send + 'static>;

pub fn spawn_reader() -> (sync::mpsc::Sender<UpdateCallback>, thread::JoinHandle<()>) {
    let (tx, rx): (sync::mpsc::Sender<UpdateCallback>, sync::mpsc::Receiver<UpdateCallback>) = sync::mpsc::channel();
    let thread = thread::Builder::new().name("desktop entry reader".to_owned()).spawn(move || {
        loop {
            let cb = rx.recv().unwrap();
            update_entries();
            cb();
        }
    }).unwrap();
    (tx, thread)
}
