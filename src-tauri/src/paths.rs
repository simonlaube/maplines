use std::path::PathBuf;
use std::fs;

pub fn maplines() -> PathBuf {
    let mut data_path = dirs::data_dir().unwrap();
    data_path.push("Maplines");
    data_path
}

pub fn tracks() -> PathBuf {
    let mut tracks_path = maplines();
    tracks_path.push("tracks");
    tracks_path
}

pub fn track(ulid: &str) -> PathBuf {
    let mut track_path = tracks();
    track_path.push(ulid);
    track_path
}

pub fn track_analysis(ulid: &str) -> PathBuf {
    let mut track_analysis = track(ulid);
    track_analysis.push("analysis.json");
    track_analysis
}

pub fn track_gpx(ulid: &str) -> PathBuf {
    let mut gpx_path = track(ulid);
    gpx_path.push("record.gpx");
    gpx_path
}

pub fn track_geojson(ulid: &str) -> PathBuf {
    let mut geojson_path = track(ulid);
    geojson_path.push("geometries.geojson");
    geojson_path
}

pub fn track_elevation(ulid: &str) -> PathBuf {
    let mut elevation_path = track(ulid);
    elevation_path.push("elevation.json");
    elevation_path
}

pub fn track_notes(ulid: &str) -> PathBuf {
    let mut notes_path = track(ulid);
    notes_path.push("notes");
    notes_path

}

// Shuttle Radar Topographic Mission
pub fn srtm() -> PathBuf {
    let mut srtm_path = maplines();
    srtm_path.push("srtm");
    srtm_path
}

pub fn create_dirs_if_not_exist() {
    fs::create_dir_all(tracks()).unwrap();
    fs::create_dir_all(srtm()).unwrap();
}