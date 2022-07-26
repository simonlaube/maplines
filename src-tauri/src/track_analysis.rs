use ulid::Ulid;
use serde::{Serialize, Deserialize};
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