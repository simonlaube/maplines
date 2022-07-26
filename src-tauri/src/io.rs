use std::fs;
use std::path::{Path, PathBuf};

pub struct MaplinesDirPaths {
    pub main: PathBuf,
    pub summary: PathBuf,
    pub geojson: PathBuf,
}
  
impl MaplinesDirPaths {
    pub fn get() -> MaplinesDirPaths {
        let mut main_path = dirs::document_dir().unwrap();
        main_path.push("maplines");
        let mut summary_path = main_path.clone();
        summary_path.push("summary");
        let mut geojson_path = main_path.clone();
        geojson_path.push("geojson");
        fs::create_dir_all(summary_path.clone()).unwrap();
        fs::create_dir_all(geojson_path.clone()).unwrap();
        MaplinesDirPaths {
            main: main_path,
            summary: summary_path,
            geojson: geojson_path,
        }
    }
}
  
#[derive(Clone)]
pub struct LinePaths {
    /// Name of the file without the path or the extension.
    pub file_name: String,
    /// Optional path to GPX file.
    gpx_path: Option<PathBuf>,
    /// Optional path to json file containing the summarised track.
    summary_path: Option<PathBuf>,
    /// Optional path to geojson file.
    geojson_path: Option<PathBuf>,
}

impl LinePaths {
  
    pub fn new_from_name(file_name: String) -> Self {
        LinePaths {
            file_name,
            gpx_path: None,
            summary_path: None,
            geojson_path: None,
        }
    }
  
    pub fn new_from_gpx(file_path: PathBuf) -> Self {
        LinePaths {
            file_name: Path::file_stem(file_path.as_path()).unwrap().to_str().unwrap().to_string(),
            gpx_path: None,
            summary_path: None,
            geojson_path: None,
        }
    }
  
    pub fn gpx_path(mut self) -> PathBuf {
        match self.gpx_path {
            Some(path) => return path.clone(),
            None => {
            self.gpx_path = Some({  let mut a: PathBuf = [dirs::document_dir().unwrap().to_str().unwrap(), 
                                    "maplines", &self.file_name].iter().collect();
                                    a.set_extension("gpx");
                                    a
                                });
            return self.gpx_path.clone().unwrap()
            }
        }
    }
  
    pub fn summary_path(mut self) -> PathBuf {
        match self.summary_path {
            Some(path) => return path.clone(),
            None => {
            self.summary_path = Some({  let mut a: PathBuf = [dirs::document_dir().unwrap().to_str().unwrap(), 
                                        "maplines", "summary", &self.file_name].iter().collect();
                                        a.set_extension("json");
                                        a
                                    });
            return self.summary_path.clone().unwrap()
            }
        }
    }
  
    pub fn geojson_path(mut self) -> PathBuf {
        match self.geojson_path {
            Some(path) => return path.clone(),
            None => {
            self.geojson_path = Some({  let mut a: PathBuf = [dirs::document_dir().unwrap().to_str().unwrap(), 
                                        "maplines", "geojson", &self.file_name].iter().collect();
                                        a.set_extension("geojson");
                                        a
                                    });
            return self.geojson_path.clone().unwrap()
            }
        }
    }

    pub fn read_geojson(mut self) -> String {
        fs::read_to_string(self.geojson_path()).unwrap().as_str().to_string()
    }
  }