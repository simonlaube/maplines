use geo::Extremes;
use geojson::{GeoJson, Feature, Value, };
use serde::{Serialize, Deserialize};
use ulid;
use gpx::{Time, Track};
use std::path::PathBuf;
use std::fs::{self, File};

use crate::paths;
/// same as Track but without links and segments
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrackAnalysis {
    pub version: i32,
    pub ulid: String,
    pub start_time: Option<String>,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub _type: Activity, // Replace with custom type struct
    pub number: Option<u32>, 
    pub creator: Option<String>,
    pub x_min: (f64, f64),
    pub x_max: (f64, f64),
    pub y_min: (f64, f64),
    pub y_max: (f64, f64),
    pub start_coords: (f64, f64),
    pub end_coords: (f64, f64),
}

impl TrackAnalysis {
    pub fn read(json_path_in: &PathBuf) -> Result<TrackAnalysis, std::io::Error> {
        let json_string = fs::read_to_string(json_path_in)?;
        let ta: TrackAnalysis = serde_json::from_str(&json_string.as_str())?;
        Ok(ta)
    }

    pub fn from_import(ulid: &ulid::Ulid, start_time: &Time, track: &Track, creator: Option<String>, geojson: GeoJson, activity: Option<Activity>) -> TrackAnalysis {

        let feature: Feature = Feature::try_from(geojson).unwrap();
        let gj_geometry: geojson::Geometry = feature.geometry.unwrap();

        // Change to Result instead of panic
        let coords = if let Value::LineString(coords) = gj_geometry.value { coords }
                                    else { panic!("could not extract coords from geojson file"); };
        let start = coords.first().unwrap();
        let start_coords: (f64, f64) = (start.first().unwrap().to_owned(), start.last().unwrap().to_owned());
        let end = coords.last().unwrap();
        let end_coords: (f64, f64) = (end.first().unwrap().to_owned(), end.last().unwrap().to_owned());
        let geo_line: geo::LineString<f64> = Value::LineString(coords).try_into().unwrap();
        let geometry: geo::Geometry = geo_line.into();
        let extremes = geometry.extremes().unwrap();
        let activity = match activity {
            None => activity_type_from_track(&track),
            Some(a) => a,
        };

        TrackAnalysis {
            version: crate::ANALYSIS_VERSION,
            ulid: ulid.to_string(),
            start_time: Some(start_time.format().unwrap()),
            name: track.name.clone(),
            comment: track.comment.clone(),
            description: track.description.clone(),
            source: track.source.clone(),
            _type: activity,
            number: track.number,
            creator: creator,
            x_min: extremes.x_min.coord.into(),
            x_max: extremes.x_max.coord.into(),
            y_min: extremes.y_min.coord.into(),
            y_max: extremes.y_max.coord.into(),
            start_coords,
            end_coords,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Activity {
    CrossCountrySkiing,
    Cycling,
    Generic, // default value
    Hiking,
    InlineSkating,
    Running,
    Swimming,
    UserDefined,
}

fn activity_type_from_track(track: &Track) -> Activity {
    match &track._type {
        Some(t) => {
            match t.as_str() {
                // Strava Activity Numbering (experimental, may possibly be inaccurate)
                "1" => Activity::Cycling,
                "4" => Activity::Hiking,
                "6" => Activity::InlineSkating,
                "7" => Activity::CrossCountrySkiing,
                "9" => Activity::Running,
                "10" => Activity::Hiking, // walk
                "16" => Activity::Swimming,
                _ => Activity::Generic,
                /*
                    1: Ride
                    2: Alpine Ski
                    3: Backcountry Ski
                    4: Hike
                    5: Ice Skate
                    6: Inline Skate
                    7: Nordic Ski
                    8: Roller Ski
                    9: Run
                    10: Walk
                    11: Workout
                    12: Snowboard
                    13: Snowshoe
                    14: Kitesurf
                    15: Windsurf
                    16: Swim
                    17: Virtual Ride
                    18: E-Bike Ride
                    19: Velomobile
                    21: Canoe
                    22: Kayaking
                    23: Rowing
                    24: Stand Up Paddling
                    25: Surfing
                    26: Crossfit
                    27: Elliptical
                    28: Rock Climb
                    29: Stair-Stepper
                    30: Weight Training
                    31: Yoga
                    51: Handcycle
                    52: Wheelchair
                    53: Virtual Run
                 */
            }
        }
        None => Activity::Generic
    }
}