use std::path::PathBuf;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrackNote {
    pub id: String,
    pub coords: (f64, f64),
    pub icon: TrackIcon,
    pub comment: Option<String>,
    pub pictures: Option<Vec<PathBuf>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TrackIcon {
    Picture,
    Text,
    Undefined,
}

pub fn get_icon_from_string(string: String) -> TrackIcon {
    match string.to_lowercase().as_str() {
        "picture" => TrackIcon::Picture,
        "text" => TrackIcon::Text,
        _ => TrackIcon::Undefined,
    }
}