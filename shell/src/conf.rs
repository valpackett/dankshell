use std::{io, fs};
use std::io::Write;
use xdg;
use ron;
use serde::Serialize;
use serde::de::DeserializeOwned;
use atomicwrites::{AtomicFile, AllowOverwrite};

#[allow(renamed_and_removed_lints)]
pub mod errors {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
            RonDeserialize(::ron::de::Error);
            RonSerialize(::ron::ser::Error);
        }
        //errors { }
    }
}

use self::errors::*;

pub struct ConfigManager {
    dirs: xdg::BaseDirectories,
}

impl ConfigManager {
    pub fn new() -> ConfigManager {
        ConfigManager {
            dirs: xdg::BaseDirectories::with_prefix("dankshell").unwrap(),
        }
    }

    pub fn read<T: DeserializeOwned + Serialize + Default>(&self, filename: &str) -> Result<T> {
        if let Some(path) = self.dirs.find_config_file(filename) {
            info!("Read config '{}' from '{:?}'", filename, path);
            return Ok(ron::de::from_reader(io::BufReader::new(fs::File::open(path)?))?)
        }
        info!("Using default config for '{:?}'", filename);
        let conf = Default::default();
        self.save(filename, &conf)?;
        Ok(conf)
    }

    pub fn save<T: Serialize>(&self, filename: &str, config: &T) -> Result<()> {
        let path = if let Some(p) = self.dirs.find_config_file(filename) { p } else {
            self.dirs.place_config_file(filename)?
        };
        info!("Writing config '{}' to '{:?}'", filename, path);
        let data = ron::ser::to_string_pretty(config, Default::default())?;
        let af = AtomicFile::new(path, AllowOverwrite);
        af.write(|f| f.write_all(data.as_bytes())).map_err(io::Error::from)?;
        Ok(())
    }
}
