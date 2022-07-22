#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

extern crate dirs;
extern crate geo;
extern crate gpx;
extern crate geojson;

mod io;
mod gps_analyzer;
mod type_converter;

use std::path::PathBuf;
use geojson::GeoJson;
use io::{MaplinesDirPaths, LinePaths};
use gps_analyzer::{GpsSummary, read_gpx};
use type_converter::write_gpx_to_geojson;
use gpx::Gpx;

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![load_gps_summaries, load_line, load_geojson])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn load_gps_summaries() -> Vec<GpsSummary> {
  let maplines_paths = MaplinesDirPaths::get();
  let gps_paths: Vec<PathBuf> = std::fs::read_dir(maplines_paths.main.clone()).unwrap()
    .into_iter()
    .map(|x| x.unwrap().path()).collect();
  let summary_paths: Vec<PathBuf> = std::fs::read_dir(maplines_paths.summary.clone()).unwrap()
    .into_iter()
    .map(|x| x.unwrap().path()).collect();
    
  let mut gps_summaries: Vec<GpsSummary> = vec![];

  for g in gps_paths {
    
    // Clone of path with json extension used for comparison.
    let mut g_json = g.clone();
    g_json.set_extension("json");

    // Gps file already analyzed.
    if summary_paths.iter().any(|a| a.as_path().file_name().unwrap().eq(g_json.as_path().file_name().unwrap())) {
      let mut json_path_in = maplines_paths.summary.clone();
      json_path_in.push(g_json.as_path().file_name().unwrap());
      gps_summaries.push(GpsSummary::from_json(json_path_in));
      continue;
    }

    // Analyze Gpx file, write summary and geojson file.
    let g_clone = g.clone();
    if g_clone.as_os_str().to_str().unwrap().ends_with(".gpx") {
      let mut geo_path = g_json;
      geo_path.set_extension("geojson");
      let file_name = String::from(geo_path.file_name().unwrap().to_str().unwrap());

      let mut geo_path = maplines_paths.geojson.clone();
      geo_path.push(file_name.clone());
      write_gpx_to_geojson(read_gpx(&g_clone), file_name, geo_path);
      
      let agf = GpsSummary::from_gpx(g);
      agf.write(maplines_paths.summary.clone());
      gps_summaries.push(agf);
      continue;
    }

    // TODO: Add support for further file types
  }

  gps_summaries
}

#[tauri::command]
fn load_line(file_name: String) -> Gpx {
  let maplines_paths = MaplinesDirPaths::get();
  let mut file_path = maplines_paths.main;
  file_path.push(file_name);
  let gpx = gps_analyzer::read_gpx(&file_path);
  gpx
}

#[tauri::command]
fn load_geojson(file_name: String) -> GeoJson {
  let mut line_paths = LinePaths::new_from_name(file_name);
  line_paths.read_geojson().parse::<GeoJson>().unwrap()
}
