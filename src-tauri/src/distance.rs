use geojson::GeoJson;
use gpx::{Gpx, Waypoint};
use geo::{Point, HaversineDistance};

use crate::{track_analysis::{TrackAnalysis, self}, io};

pub fn from_track_analysis(track_analysis: &TrackAnalysis) -> Result<f64, String> {
    let gpx = io::read_gpx(&track_analysis.ulid);
    // let gpx = io::read_gpx(&track_analysis.ulid);
    match gpx {
        Some(g) => {
            let gpx = g;
            return Ok(from_gpx(&gpx));

            // return find_time_jumps(&gpx, min_break_time).unwrap()
        }
        _ => Err("Could not calculate distance".to_string())
    }
}

pub fn from_gpx(gpx: &Gpx) -> f64 {
    let mut last_p: &Waypoint = &gpx.tracks[0].segments[0].points[0].clone();
    let mut dist = 0.0;
    for p in &gpx.tracks[0].segments[0].points {
        let d = last_p.point().haversine_distance(&p.point().into());
        if d < 40.0 {
            dist += d;
        }
        last_p = p;
    }
    return dist
}