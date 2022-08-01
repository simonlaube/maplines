use std::str::FromStr;

use geo::{Coordinate, Point};
use gpx::{Gpx, Time, Waypoint};
use chrono;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{track_analysis::TrackAnalysis, io};

/// Returns an Option containing the points before and after the break
/// and the time passed in seconds.
pub fn find(track_analysis: TrackAnalysis) -> Option<Vec<Pause>> {
    let min_break_time: u64 = 120;
    let geojson = io::read_geojson(&track_analysis.ulid);
    let gpx = io::read_gpx(&track_analysis.ulid);
    match (geojson, gpx) {
        (Some(gj), Some(g)) => {
            let geojson = gj;
            let gpx = g;
            println!("{:?}", find_time_jumps(&gpx, min_break_time));
            return find_time_jumps(&gpx, min_break_time).unwrap()
        }
        _ => return None
    }

    None
}

// TODO: Error handling
// TODO: check that distance is "short"
/// Finds consecutive points having long time-gap and short distance
fn find_time_jumps(gpx: &Gpx, min_break_time: u64) -> Result<Option<Vec<Pause>>, String> {
    let mut prev_time: Option<Time> = None;
    let mut prev_point: Option<&Waypoint> = None;
    let mut result: Vec<Pause> = vec![];
    for p in &gpx.tracks[0].segments[0].points {
        (prev_time, prev_point) = match (prev_time, p.time) {
            (Some(pt), Some(t)) => {
                let diff = OffsetDateTime::from(t).unix_timestamp() - OffsetDateTime::from(pt).unix_timestamp();
                if diff < 0 {
                    // Error
                }
                if diff as u64 >= min_break_time {
                    result.push(Pause { point_before: prev_point.unwrap().point().clone().into(), point_after: p.point().clone().into(), duration_sec: diff as u64 });
                }
                (Some(t), Some(p))
            },
            (None, Some(t)) => (Some(t), Some(p)),
            (_, None) => return Err("Corrupted timestamps in gpx file".to_string())
        }
    }
    Ok(Some(result))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pause {
    pub point_before: (f64, f64),
    pub point_after: (f64, f64),
    pub duration_sec: u64,
}