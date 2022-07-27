use gpx::Track;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;
/// same as Track but without links and segments
#[derive(Serialize, Deserialize, Debug)]
pub struct TrackAnalysis {
    pub version: i32,
    pub ulid: String,
    pub start_time: Option<String>,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub _type: Option<String>, // Replace with custom type struct
    pub number: Option<u32>, 
    pub creator: Option<String>,
    pub x_min: (f64, f64),
    pub x_max: (f64, f64),
    pub y_min: (f64, f64),
    pub y_max: (f64, f64),
}

impl TrackAnalysis {
    pub fn from_json_path(json_path_in: &PathBuf) -> Result<TrackAnalysis, std::io::Error> {
        let json_string = fs::read_to_string(json_path_in)?;
        let ta: TrackAnalysis = serde_json::from_str(&json_string.as_str())?;
        Ok(ta)
    }
}