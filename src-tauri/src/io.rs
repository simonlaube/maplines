use std::{path::PathBuf, fs::{self, File}, io::BufReader};

use geojson::GeoJson;
use gpx::{Gpx, read};

use crate::{paths, track_analysis::TrackAnalysis};

pub fn read_geojson(ulid: &String) -> Option<GeoJson> {
    let mut path = paths::geojson();
    path.push(ulid);
    path.set_extension("geojson");

    match fs::read_to_string(path) {
        Ok(s) => {
            match s.parse::<GeoJson>() {
                Ok(g) => Some(g),
                _ => None,
            }
        },
        _ => None,
    }
}

pub fn read_gpx(ulid: &String) -> Option<Gpx> {
    let mut path = paths::gpx();
    path.push(ulid);
    path.set_extension("gpx");

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut gpx = read(reader); // TODO: remove unwrap
    match gpx {
        Ok(g) => Some(g),
        _ => None
    }
}

pub fn read_track_analysis(ulid: &String) -> Option<TrackAnalysis> {
    let mut path = paths::track_analysis();
    path.push(ulid);
    path.set_extension("json");
    Some(TrackAnalysis::read(&path).unwrap())
}