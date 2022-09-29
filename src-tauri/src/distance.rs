use gpx::{Gpx, Waypoint};
use geo::{HaversineDistance};

use crate::pause::Pause;
/*
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
*/

pub fn calculate(gpx: &Gpx, pauses: &Vec<Pause>) -> f64 {
    let mut last_p: &Waypoint = &gpx.tracks[0].segments[0].points[0].clone();
    let mut dist = 0.0;

    let mut pause_pos = 0;

    for (i, p) in gpx.tracks[0].segments[0].points.iter().enumerate() {
        // distance after last pause to end
        if pause_pos == pauses.len() {
            dist += last_p.point().haversine_distance(&p.point().into());
            last_p = p;
        }
        // Line until pause start
        else if i < pauses.get(pause_pos).unwrap().index_before {
            dist += last_p.point().haversine_distance(&p.point().into());
            last_p = p;
        }
        // End line and start unedited pause line
        else if i == pauses.get(pause_pos).unwrap().index_before {
            dist += last_p.point().haversine_distance(&p.point().into());
            last_p = p;
        }
        // Unedited line until pause end
        else if i < pauses.get(pause_pos).unwrap().index_after {
            // do nothing distance of pause cluster not counted
        }
        // End unedited pause line, add direct pause line, start new line, remove first pause from vector
        else {
            // Do not add pause distance to total distance
            // dist += last_p.point().haversine_distance(&p.point().into());
            last_p = p; // add distance from pause start pos to pause end pos

            pause_pos += 1;
        }
    }
    return dist
}