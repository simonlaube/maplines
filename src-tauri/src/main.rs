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
// extern crate geotiff;
extern crate reqwest;
extern crate tokio;

// used by geotiff
extern crate byteorder;
#[macro_use]
extern crate enum_primitive;
extern crate num;
extern crate zip;

mod import;
mod io;
mod geotiff;
mod track_analysis;
mod track_note;
mod line;
mod paths;
mod pause;
mod errors;
mod settings;
mod util;
mod elevation;
mod distance;

use std::ffi::OsStr;
use std::path::PathBuf;
use geojson::GeoJson;
use pause::Pause;
use track_analysis::TrackAnalysis;
use settings::Settings;
use tauri::api::{dialog};
use tauri::{CustomMenuItem, Menu, Submenu};
use track_note::TrackNote;
use ulid::Ulid;

const ANALYSIS_VERSION: i32 = 1;
const SETTINGS_VERSION: i32 = 1;

fn main() {
  paths::create_dirs_if_not_exist();
  //let mut settings: Mutex<Settings> = Mutex::new(Settings::load().unwrap());
  let version_item = CustomMenuItem::new("version".to_string(), "Version");
  let main_menu = Submenu::new("Main", Menu::new()
  .add_item(version_item));
  let import_gpx = CustomMenuItem::new("gpx".to_string(), "Import GPX Files...");
  let import_fit = CustomMenuItem::new("fit".to_string(), "Import FIT Files...");
  let import_path = CustomMenuItem::new("path".to_string(), "Add Import Path...");
  let import_direct = CustomMenuItem::new("direct".to_string(), "Import From Paths");
  let open_menu = Submenu::new("Open", Menu::new().add_item(import_gpx).add_item(import_fit).add_item(import_path).add_item(import_direct));

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
                let track_analysis = match import::gpx(&fp) {
                  Err(_e) => { println!("File with same start time is already present."); return; },
                  Ok(t) => t,
                };
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
      "path" => {
        dialog::FileDialogBuilder::default().pick_folder(|dir_path| {
          let mut settings = Settings::load().unwrap();
          settings.add_path(dir_path.unwrap());
        });

      }
      "direct" => {
        let settings = Settings::load().unwrap();
        std::thread::spawn(move || {
          for p in settings.import_paths {
            // import files
            let paths: Vec<PathBuf> = std::fs::read_dir(p).unwrap()
            .into_iter()
            .map(|x| x.unwrap().path())
            //.filter(|x| x.file_name().unwrap().to_str().unwrap().ends_with(".json"))
            .collect();
            for p in paths {
              if p.extension() == Some(OsStr::new("fit")) || p.extension() == Some(OsStr::new("FIT")) {
                
                match import::fit(&p) {
                  Ok(ta) => event.window().emit("track_import", ta).unwrap(),
                  _ => ()
                }
                
              } else if p.extension() == Some(OsStr::new("gpx")) {
                let track_analysis = import::gpx(&p).unwrap();
                event.window().emit("track_import", track_analysis).unwrap();
              }
            }
  
          }
        });
      }
      _ => {}
    })
    .invoke_handler(tauri::generate_handler![load_geojson, load_pauses, load_track_analysis, calculate_pauses, load_track_display_data, save_track_changes, load_elevation, load_notes, join_tracks, delete_track, recalculate_track, add_note])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");

}

#[tauri::command]
fn load_track_analysis() -> Vec<TrackAnalysis> {
  let paths: Vec<PathBuf> = std::fs::read_dir(paths::tracks()).unwrap()
  .into_iter()
  .map(|x| x.unwrap().path())
  // .filter(|x| x.file_name().unwrap().to_str().unwrap().ends_with(".json"))
  .filter(|x| x.is_dir())
  .collect();
  let mut result: Vec<TrackAnalysis> = vec![];
  for p in paths {
    /*match TrackAnalysis::read(&p) {
      Ok(ta) => result.push(ta),
      _ => println!("Could not load track with path {:?}.", p),
    }*/
    match io::read_track_analysis(&p.file_name().unwrap().to_str().unwrap().to_string()) {
      Ok(ta) => result.push(ta),
      _ => println!("Could not load track with path {:?}.", p),
    }
  }
  result
}

#[tauri::command]
fn load_geojson(ulid: String) -> Option<GeoJson> {
  io::read_geojson(&ulid)
/*
  match fs::read_to_string(line_path) {
    Ok(s) => {
      match s.parse::<GeoJson>() {
        Ok(g) => Some(g),
        _ => None,
      }
    },
    _ => None,
  }
  */
}
#[tauri::command]
fn load_pauses(ulid: String) -> Vec<Pause> {
  pause::find(&io::read_gpx(&ulid).unwrap())
}

#[tauri::command]
fn calculate_pauses(ulid: String) -> Option<(Vec<Pause>, GeoJson)> {
  // let gpx = io::read_gpx(&ulid).unwrap();
  // elevation::from_latlong(gpx);
  // elevation::from_latlong(gpx);
  // distance::from_track_analysis(&io::read_track_analysis(&ulid).unwrap());
  // println!("done");

  // let pauses = pause::find(io::read_track_analysis(&ulid).unwrap());
  let pauses = load_pauses(ulid.clone());
  let lines = line::arrange_display(&io::read_gpx(&ulid).unwrap(), None, Some(&pauses));
  Some((pauses, lines))
}

#[tauri::command]
fn load_track_display_data(ulid: String) -> Option<(Vec<Pause>, GeoJson)> {
  // TODO: if geojson or pauses do not exist -> create them
  let geojson = io::read_geojson(&ulid).unwrap();
  let track_analysis = io::read_track_analysis(&ulid).unwrap();
  Some((track_analysis.pauses, geojson))
}

#[tauri::command]
fn load_elevation(ulid: String) -> Option<(Vec<(f64, f64)>, Vec<(f64, f64)>)> {
  if paths::track_elevation(&ulid).exists() {
    return Some(io::read_elevation(&ulid).unwrap());
  }
  None
}

#[tauri::command]
fn load_notes(ulid: String) -> Option<Vec<TrackNote>> {
  io::read_track_notes(&ulid)
}

#[tauri::command]
fn add_note(ulid: String, coords: (f64, f64), icon: String, comment: Option<String>, img_paths: Option<Vec<String>>) {
  let img_paths = match img_paths {
    Some(p) => Some(p.iter().map(|p| PathBuf::from(p)).collect()),
    None => None
  };
  let new_note = TrackNote {
    id: Ulid::new().to_string(), // no time associated with note
    coords,
    icon: track_note::get_icon_from_string(icon),
    comment,
    pictures: img_paths
  };
  println!("{:?}", new_note);
  return;

  let mut notes;
  match io::read_track_notes(&ulid) {
    Some(n) => {
      notes = n;
      notes.push(new_note);
    },
    None => notes = vec![new_note],
  }
  io::write_track_notes(notes, &ulid).unwrap();
}

#[tauri::command]
fn save_track_changes(ulid: String, name: String, activity: String) {
  println!("{}", activity);
  let mut track_analysis = io::read_track_analysis(&ulid).unwrap();
  track_analysis.name = Some(name.clone());
  track_analysis._type = track_analysis::activity_type_from_string(&activity);
  io::write_track_analysis(&track_analysis).unwrap();
  // gpx writing takes longer and is therefore handled in separate thread
  std::thread::spawn(move || {
    println!("{} : {} : {}", ulid, name, activity);
    let mut gpx = io::read_gpx(&ulid).unwrap();
    gpx.tracks[0].name = Some(name);
    io::write_gpx(&gpx, &ulid);
  });
}

#[tauri::command]
fn delete_track(ulid: String) {
  // println!("remove {:?}", paths::track(ulid));
  std::fs::remove_dir_all(paths::track(&ulid));
}

#[tauri::command]
fn join_tracks(ulids: Vec<String>) {
  util::join_tracks(ulids);
}

#[tauri::command]
fn recalculate_track(ulid: String) {
  util::recalculate_track(ulid);
}