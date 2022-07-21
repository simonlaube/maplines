#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod gps_analyzer;
extern crate dirs;
extern crate gpx;
extern crate geo;

use std::path::PathBuf;
use std::fs;
use gps_analyzer::AnalyzedGpsFile;
use gpx::Gpx;


fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![load_analyzed_gpx_files, load_line])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn load_analyzed_gpx_files() -> Vec<AnalyzedGpsFile> {
  let maplines_paths = MaplinesDirPaths::get();
  let gps_paths: Vec<PathBuf> = std::fs::read_dir(maplines_paths.main.clone()).unwrap()
    .into_iter()
    .map(|x| x.unwrap().path()).collect();
  let json_paths: Vec<PathBuf> = std::fs::read_dir(maplines_paths.analyzed.clone()).unwrap()
    .into_iter()
    .map(|x| x.unwrap().path()).collect();
    
  let mut analyzed_gps_files: Vec<AnalyzedGpsFile> = vec![];

  for g in gps_paths {
    
    // Clone of path with json extension used for comparison.
    let mut g_json = g.clone();
    g_json.set_extension("json");

    // Gps file already analyzed.
    if json_paths.iter().any(|j| j.as_path().file_name().unwrap().eq(g_json.as_path().file_name().unwrap())) {
      let mut json_path_in = maplines_paths.analyzed.clone();
      json_path_in.push(g_json.as_path().file_name().unwrap());
      analyzed_gps_files.push(AnalyzedGpsFile::from_json(json_path_in));
      continue;
    }

    // Gpx file has to be analyzed and written to new file.
    if g.as_os_str().to_str().unwrap().ends_with(".gpx") {
      let agf = AnalyzedGpsFile::from_gpx(g);
      agf.write(maplines_paths.analyzed.clone());
      analyzed_gps_files.push(agf);
      continue;
    }

    // TODO: Add support for further file types
  }

  analyzed_gps_files
}

#[tauri::command]
fn load_line(file_name: String) -> Gpx {
  let maplines_paths = MaplinesDirPaths::get();
  let mut file_path = maplines_paths.main;
  file_path.push(file_name);
  let gpx = gps_analyzer::read_gpx(&file_path);
  gpx
}

struct MaplinesDirPaths {
  main: PathBuf,
  analyzed: PathBuf,
}

impl MaplinesDirPaths {
  fn get() -> MaplinesDirPaths {
    let mut main_path = dirs::document_dir().unwrap();
    main_path.push("maplines");
    let mut analyzed_path = main_path.clone();
    analyzed_path.push("analyzed");
    fs::create_dir_all(analyzed_path.clone());
    MaplinesDirPaths { main: main_path, analyzed: analyzed_path }
  }
}
