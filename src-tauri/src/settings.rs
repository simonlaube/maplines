use std::path::PathBuf;
use std::fs::{File, self};
use std::io::{self, Write};

use serde::{Serialize, Deserialize};

use crate::paths;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub version: i32,
    pub import_paths: Vec<PathBuf>,
}

impl Settings {
    fn new() -> Settings {
        Settings {
            version: crate::SETTINGS_VERSION,
            import_paths: vec![],
        }
    }
    pub fn load() -> Result<Settings, std::io::Error> {
        let mut path = paths::maplines();
        path.push("settings");
        path.set_extension("json");
        if !path.exists() {
            let mut settings = Settings::new();
            settings.write();
            return Ok(settings)
        }
        let json_string = fs::read_to_string(path)?;
        let s: Settings = serde_json::from_str(&json_string.as_str())?;
        Ok(s)
    }

    pub fn add_path(&mut self, path: PathBuf) {
        self.import_paths.push(path);
        self.write();
    }

    fn write(&self) -> Result<(), io::Error> {
        let mut path = paths::maplines();
        path.push("settings");
        path.set_extension("json");
        let mut file = File::create(path).unwrap();
        write!(file, "{}", serde_json::to_string(self)?)?;
        Ok(())
    }
}