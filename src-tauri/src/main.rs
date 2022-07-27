#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

extern crate dirs;
extern crate geo;
extern crate gpx;
extern crate geojson;
extern crate time;

mod io;
mod import;
mod gps_analyzer;
mod track_analysis;
mod type_converter;
mod paths;

use std::path::PathBuf;
use geojson::GeoJson;
use io::{MaplinesDirPaths, LinePaths};
use gps_analyzer::{GpsSummary, read_gpx};
use type_converter::write_gpx_to_geojson;
use gpx::Gpx;
use tauri::api::dialog;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};

const ANALYSIS_VERSION: i32 = 1;

fn main() {
  paths::create_dirs_if_not_exist();
  let version_item = CustomMenuItem::new("version".to_string(), "Version");
  let main_menu = Submenu::new("Main", Menu::new()
  .add_item(version_item));
  //let menu = Menu::new().add_item(open);
    //.add_submenu(fileMenu)
    //.add_native_item(MenuItem::Separator)
    //.add_native_item(MenuItem::Quit);
  let import_gpx = CustomMenuItem::new("gpx".to_string(), "Import GPX Files");
  let import_direct = CustomMenuItem::new("direct".to_string(), "Import from GPS Device");
  let mut open_menu = Submenu::new("Open", Menu::new().add_item(import_gpx).add_item(import_direct));

  tauri::Builder::default()
    .menu(Menu::new().add_submenu(main_menu).add_submenu(open_menu))
    .on_menu_event(|event| match event.menu_item_id() {
      "version" => {
        println!("{}", option_env!("CARGO_PKG_VERSION").unwrap());
      }
      "gpx" => {
        dialog::FileDialogBuilder::default()
          .add_filter("GPS", &["gpx"])
          .pick_file(|path_buf| match path_buf {
            Some(p) => {
              import::gpx(&p).unwrap();
            }
            _ => {}
          });
      }
      "direct" => {
        dialog::FileDialogBuilder::default()
          .add_filter("GPS", &["fit", "FIT"])
          .pick_file(|path_buf| match path_buf {
            Some(p) => {}
            _ => {}
          });
      }
      _ => {}
    })
    .invoke_handler(tauri::generate_handler![load_gps_summaries, load_line, load_geojson])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  paths::create_dirs_if_not_exist();

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
