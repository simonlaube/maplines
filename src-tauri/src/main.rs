#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

extern crate dirs;
extern crate chrono;
extern crate geo;
extern crate gpx;
extern crate geojson;
extern crate time;

mod import;
mod track_analysis;
mod type_converter;
mod paths;
mod errors;

use std::fs;
use std::path::PathBuf;
use geojson::GeoJson;
use track_analysis::TrackAnalysis;
use tauri::api::dialog;
use tauri::{CustomMenuItem, Menu, Submenu};

const ANALYSIS_VERSION: i32 = 1;

fn main() {
  paths::create_dirs_if_not_exist();
  let version_item = CustomMenuItem::new("version".to_string(), "Version");
  let main_menu = Submenu::new("Main", Menu::new()
  .add_item(version_item));
  let import_gpx = CustomMenuItem::new("gpx".to_string(), "Import GPX Files...");
  let import_fit = CustomMenuItem::new("fit".to_string(), "Import FIT Files...");
  let import_direct = CustomMenuItem::new("direct".to_string(), "Import from GPS Device");
  let mut open_menu = Submenu::new("Open", Menu::new().add_item(import_gpx).add_item(import_fit).add_item(import_direct));

  tauri::Builder::default()
    .menu(Menu::new().add_submenu(main_menu).add_submenu(open_menu))
    .on_menu_event(|event| match event.menu_item_id() {
      "version" => {
        println!("{}", option_env!("CARGO_PKG_VERSION").unwrap());
      }
      "gpx" => {
        dialog::FileDialogBuilder::default()
          .add_filter("GPS", &["gpx"])
          .pick_files(move |file_paths| {
            match file_paths {
              Some(vec_fp) => {
                for fp in vec_fp {
                  let track_analysis = import::gpx(&fp).unwrap();
                  event.window().emit("track_import", track_analysis).unwrap();
                }
              }
              _ => { dbg!("gpx file could not be imported."); },
            }
          })
      }
      "fit" => {
        dialog::FileDialogBuilder::default()
          .add_filter("FIT", &["FIT", "fit"])
          .pick_files(move |file_paths| {
            match file_paths {
              Some(vec_fp) => {
                for fp in vec_fp {
                  let track_analysis = import::fit(&fp).unwrap();
                  event.window().emit("track_import", track_analysis).unwrap();
                }
              }
              _ => { dbg!("gpx file could not be imported."); },
            }
          })
      }
      "direct" => {
        dialog::FileDialogBuilder::default()
          .add_filter("GPS", &["fit", "FIT"])
          .pick_file(|path_buf| match path_buf {
            Some(p) => {}
            _ => {}
          })
      }
      _ => {}
    })
    .invoke_handler(tauri::generate_handler![load_geojson, load_track_analysis])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
  paths::create_dirs_if_not_exist();

}

#[tauri::command]
fn load_track_analysis() -> Vec<TrackAnalysis> {
  let paths: Vec<PathBuf> = std::fs::read_dir(paths::track_analysis()).unwrap()
    .into_iter()
    .map(|x| x.unwrap().path())
    .filter(|x| x.file_name().unwrap().to_str().unwrap().ends_with(".json"))
    .collect();
  let mut result: Vec<TrackAnalysis> = vec![];
  for p in paths {
    match TrackAnalysis::read(&p) {
      Ok(ta) => result.push(ta),
      _ => println!("Could not load track with path {:?}.", p),
    }
  }
  result
}

#[tauri::command]
fn load_geojson(ulid: String) -> Option<GeoJson> {
  let mut line_path = paths::geojson();
  line_path.push(ulid);
  line_path.set_extension("geojson");

  match fs::read_to_string(line_path) {
    Ok(s) => {
      match s.parse::<GeoJson>() {
        Ok(g) => Some(g),
        _ => None,
      }
    },
    _ => None,
  }
}
