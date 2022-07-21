use geo;
use gpx;
use serde::{Serialize, Deserialize};

use std::io::BufReader;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;

use gpx::read;
use gpx::{Gpx, Track, TrackSegment};

/// same as Track but without links and segments
#[derive(Serialize, Deserialize, Debug)]
pub struct AnalyzedGpsFile {
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub _type: Option<String>,
    pub number: Option<u32>,
    pub creator: Option<String>,

    // Maximal elevation of track
    // pub max_ele: f32,

    // Minimal elevation of track
    // pub min_ele: f32,

    // Total length of track in metres
    // pub length: f32,

    // Initial elevation of track
    // pub begin_ele: f32,

    // End elevation of track
    // pub end_ele: f32,

    // Maximum latitude of track
    // pub north: f32,

    // Maximum longitude of track
    // pub east: f32,

    // Minimum latitude of track
    // pub south: f32,

    // Minimum longitude of track
    // pub west: f32,

    // List containing all local maxima (determined by yet do be defined algorithm)
    // pub local_maxima: Option<Vec<gpx::Waypoint>>,

    // time

    // moving time

    // inter-segment-dist

}

impl AnalyzedGpsFile {

    pub fn from_gpx(gpx_path_in: PathBuf) -> AnalyzedGpsFile {
        
        let gpx = read_gpx(&gpx_path_in);
        let track: Track = gpx.tracks[0].clone();
        let file_name = gpx_path_in.file_name().unwrap();
        let file_path = gpx_path_in.to_str().unwrap().to_string();

        AnalyzedGpsFile {
            file_name: Some(String::from(file_name.to_str().unwrap())),
            file_path: Some(file_path),
            name: track.name,
            comment: track.comment,
            description: track.description,
            source: track.source,
            _type: track._type,
            number: track.number,
            creator: gpx.creator,
        }
    }
    
    pub fn from_json(json_path_in: PathBuf) -> AnalyzedGpsFile {
        let agf: AnalyzedGpsFile = serde_json::from_str(fs::read_to_string(json_path_in).unwrap().as_str()).unwrap();
        agf
    }

    pub fn write(&self, analized_path: PathBuf) {
        let mut path = analized_path.clone();
        path.push(self.file_name.clone().unwrap());
        path.set_extension("json");
        let mut file = File::create(path).unwrap();
        let json = serde_json::to_string(self).unwrap();
        write!(file, "{}", json);
    }
}

pub fn read_gpx(gpx_path_in: &PathBuf) -> Gpx {
    let file = File::open(gpx_path_in).unwrap();
    let reader = BufReader::new(file);
    
    let gpx: Gpx = read(reader).unwrap();
    gpx
}

pub fn gpx_to_geojson(gpx: Gpx) {
    todo!();
}