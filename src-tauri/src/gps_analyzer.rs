use gpx;
use geo::{self, Extremes};
use serde::{Serialize, Deserialize};

use std::io::BufReader;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::io::Write;

use crate::io::LinePaths;
use geojson::{GeoJson, Value, Feature};
use gpx::read;
use gpx::{Gpx, Track, TrackSegment};

/// same as Track but without links and segments
#[derive(Serialize, Deserialize, Debug)]
pub struct GpsSummary {
    pub file_name: Option<String>,
    pub gpx_path: Option<String>,
    pub geojson_path: Option<String>,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub _type: Option<String>,
    pub number: Option<u32>,
    pub creator: Option<String>,
    pub x_min: (f64, f64),
    pub x_max: (f64, f64),
    pub y_min: (f64, f64),
    pub y_max: (f64, f64),

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

impl GpsSummary {

    pub fn from_gpx(gpx_path_in: PathBuf) -> GpsSummary {
        
        let gpx = read_gpx(&gpx_path_in);
        let track: Track = gpx.tracks[0].clone();
        let line_paths = LinePaths::new_from_gpx(gpx_path_in.clone());
        let gpx_path = gpx_path_in.to_str().unwrap().to_string();
        let geojson_path = line_paths.clone().geojson_path().to_str().unwrap().to_string();
        
        // analyze geo data
        let geojson_line: GeoJson = line_paths.clone().read_geojson().parse::<GeoJson>().unwrap();
        let feature: Feature = Feature::try_from(geojson_line).unwrap();
        let geom: geojson::Geometry = feature.geometry.unwrap();
        let coords = if let Value::LineString(coords) = geom.value { coords }
            else { panic!("could not extract coords from geojson file"); };
        let geo_line: geo::LineString<f64> = Value::LineString(coords).try_into().unwrap();
        let geometry: geo::Geometry = geo_line.into();
        let extremes = geometry.extremes().unwrap();



        GpsSummary {
            file_name: Some(line_paths.file_name.clone()),
            gpx_path: Some(gpx_path),
            geojson_path: Some(geojson_path),
            name: track.name,
            comment: track.comment,
            description: track.description,
            source: track.source,
            _type: track._type,
            number: track.number,
            creator: gpx.creator,
            x_min: extremes.x_min.coord.into(),
            x_max: extremes.x_max.coord.into(),
            y_min: extremes.y_min.coord.into(),
            y_max: extremes.y_max.coord.into(),
        }
    }
    
    pub fn from_json(json_path_in: PathBuf) -> GpsSummary {
        let agf: GpsSummary = serde_json::from_str(fs::read_to_string(json_path_in).unwrap().as_str()).unwrap();
        agf
    }

    pub fn write(&self, mut summary_path: PathBuf) {
        summary_path.push(self.file_name.clone().unwrap());
        summary_path.set_extension("json");
        let mut file = File::create(summary_path).unwrap();
        let json = serde_json::to_string(self).unwrap();
        write!(file, "{}", json).unwrap();
    }
}

pub fn read_gpx(gpx_path_in: &PathBuf) -> Gpx {
    let file = File::open(gpx_path_in).unwrap();
    let reader = BufReader::new(file);
    
    let gpx: Gpx = read(reader).unwrap();
    gpx
}