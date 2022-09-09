use std::path::PathBuf;
use std::fs;

pub fn maplines() -> PathBuf {
    let mut data_path = dirs::data_dir().unwrap();
    data_path.push("Maplines");
    data_path
}

pub fn track_analysis() -> PathBuf {
    let mut track_analysis = maplines();
    track_analysis.push("track_analysis");
    track_analysis
}

pub fn gpx() -> PathBuf {
    let mut gpx_path = maplines();
    gpx_path.push("gpx");
    gpx_path
}

pub fn geojson() -> PathBuf {
    let mut geojson_path = maplines();
    geojson_path.push("geojson");
    geojson_path
}

// Shuttle Radar Topographic Mission
pub fn srtm() -> PathBuf {
    let mut srtm_path = maplines();
    srtm_path.push("srtm");
    srtm_path
}

pub fn create_dirs_if_not_exist() {
    fs::create_dir_all(track_analysis()).unwrap();
    fs::create_dir_all(gpx()).unwrap();
    fs::create_dir_all(geojson()).unwrap();
    fs::create_dir_all(srtm()).unwrap();
}