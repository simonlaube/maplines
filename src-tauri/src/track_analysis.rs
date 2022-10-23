use geo::Extremes;
use geojson::{GeoJson, Feature, Value, };
use serde::{Serialize, Deserialize};
use time::OffsetDateTime;
use ulid::{self, Ulid};
use gpx::{Time, Track, Gpx};
use std::path::PathBuf;
use std::fs;
use std::time::Duration;

use crate::{distance, elevation, io};
use crate::pause::{self, Pause};
/// same as Track but without links and segments
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TrackAnalysis {
    pub version: i32,
    pub ulid: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub time_moving: Option<u64>,
    pub time_total: Option<u64>,
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
    pub distance: f64, // in kilometers
    pub avg_vel: Option<f64>, // in kph
    pub ele_gain: Option<f64>,
    pub ele_loss: Option<f64>,
    pub ele_max: Option<f64>,
    pub ele_min: Option<f64>,
    pub pauses: Vec<Pause>,
}

impl TrackAnalysis {
    pub fn read(json_path_in: &PathBuf) -> Result<TrackAnalysis, std::io::Error> {
        let json_string = fs::read_to_string(json_path_in)?;
        let ta: TrackAnalysis = serde_json::from_str(&json_string.as_str())?;
        Ok(ta)
    }

    pub fn new(ulid: Option<String>, geojson: &GeoJson, gpx: &Gpx, activity: Option<Activity>) -> TrackAnalysis {

        let track: Track = gpx.tracks[0].clone();

        let feature: Feature = Feature::try_from(geojson.clone()).unwrap();
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
        let pauses: Vec<Pause> = pause::find(gpx);
        let distance = distance::calculate(gpx, &pauses);
        let start_time = gpx.tracks[0].segments[0].points[0].time.unwrap();
        let start_odt: OffsetDateTime = start_time.into();
        let end_time = gpx.tracks[0].segments[0].points.last().unwrap().time.unwrap();
        let end_odt: OffsetDateTime = end_time.into();
        let time_total = (end_odt.unix_timestamp() - start_odt.unix_timestamp()).abs() as u64;
        let time_moving = time_total - pauses.iter().map(|x| x.duration_sec).sum::<u64>();
        let avg_vel: f64 = (distance / 1000.) / (time_moving as f64 / 3600.);
        println!("avg_vel: {}", avg_vel);

        // let duration = Duration::from_secs(secs)
        // let time_total: std::time::Duration = end_time. - start_time;

        let ulid = match ulid {
            Some(u) => u,
            None => Ulid::from_datetime(start_time.into()).to_string(),
        };

        let (ele, ele_gain, ele_loss, ele_max, ele_min, coords) = match elevation::from_latlong(gpx, &pauses) {
            Ok(e) => e,
            Err(e) => {
                println!("{:?}", e);
                (vec![], 0., 0., 0., 0., vec![])
            }
        };
        io::write_elevation(ele, coords, &ulid);

        TrackAnalysis {
            version: crate::ANALYSIS_VERSION,
            ulid: ulid,
            start_time: Some(start_time.format().unwrap()),
            end_time: Some(end_time.format().unwrap()),
            time_total: Some(time_total),
            time_moving: Some(time_moving),
            name: track.name.clone(),
            comment: track.comment.clone(),
            description: track.description.clone(),
            source: track.source.clone(),
            _type: activity,
            number: track.number,
            creator: gpx.creator.clone(),
            x_min: extremes.x_min.coord.into(),
            x_max: extremes.x_max.coord.into(),
            y_min: extremes.y_min.coord.into(),
            y_max: extremes.y_max.coord.into(),
            start_coords,
            end_coords,
            distance,
            avg_vel: Some(avg_vel),
            ele_gain: Some(ele_gain),
            ele_loss: Some(ele_loss),
            ele_max: Some(ele_max),
            ele_min: Some(ele_min),
            pauses,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Activity {
    XCSkiing,
    Cycling,
    Hiking,
    InlineSkating,
    Running,
    Swimming,
    Other, // Default Value
}

fn activity_type_from_track(track: &Track) -> Activity {
    match &track._type {
        Some(t) => {
            match t.as_str() {
                // Strava Activity Numbering (experimental, may possibly be inaccurate)
                "1" => Activity::Cycling,
                "4" => Activity::Hiking,
                "6" => Activity::InlineSkating,
                "7" => Activity::XCSkiing,
                "9" => Activity::Running,
                "10" => Activity::Hiking, // walk
                "16" => Activity::Swimming,
                _ => Activity::Other,
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
        None => Activity::Other
    }
}

pub fn activity_type_from_string(activity: &String) -> Activity {
    match activity.to_lowercase().as_str() {
        "xcskiing" => Activity::XCSkiing,
        "cycling" => Activity::Cycling,
        "hiking" => Activity::Hiking,
        "inlineskating" => Activity::InlineSkating,
        "running" => Activity::Running,
        "swimming" => Activity::Swimming,
        _ => Activity::Other,
    }
}