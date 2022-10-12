use std::cmp::Ordering;
use ulid::Ulid;
use gpx::Track;

use crate::io::{self, write_track_analysis, write_geojson, write_gpx};
use crate::line::arrange_display;
use crate::track_analysis::{TrackAnalysis, self};
use crate::{pause, elevation};


pub fn track_with_start_time_exists(start_time: &String) -> bool {
    let track_analysis = crate::load_track_analysis();
    for t in track_analysis {
        if t.start_time.eq(&Some(start_time.clone())) {
            return true;
        }
    }
    false
}

pub fn join_tracks(ulids: Vec<String>) -> Option<()> {
    let mut analysis: Vec<TrackAnalysis> = ulids.iter().map(|x| io::read_track_analysis(x).unwrap()).collect();
    // TODO: check if this sorting always works correctly
    analysis.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
    
    // TODO: check if tracks do not overlap in time
    /*
    let mut last_end_time = analysis[0].end_time.clone().unwrap();
    analysis.remove(0);
    for a in analysis {
        if a.start_time.clone().unwrap().partial_cmp(&last_end_time).unwrap() == Ordering::Greater {
            println!("something went wrong ordering the tracks");
            return None;
        }
        println!("{:?}", a.start_time);
        last_end_time = a.end_time.clone().unwrap();
    }
    */

    let mut new_gpx = io::read_gpx(&analysis[0].ulid).unwrap();
    analysis.remove(0);
    for a in analysis {
        let next_part = io::read_gpx(&a.ulid).unwrap();
        for p in &next_part.tracks[0].segments[0].points {
            new_gpx.tracks[0].segments[0].points.push(p.clone());
        }
    }
    let geojson = arrange_display(&new_gpx, None, None);
    
    // analyze geo data
    let track_analysis = TrackAnalysis::new(None, &geojson, &new_gpx, None);
    let geojson = arrange_display(&new_gpx, Some(geojson), Some(&track_analysis.pauses));
    write_track_analysis(&track_analysis).unwrap();
    write_geojson(&geojson, &track_analysis.ulid).unwrap();
    write_gpx(&new_gpx, &track_analysis.ulid).unwrap();

    println!("done");
    Some(())
}

pub fn recalculate_track(ulid: String) {
    let gpx = io::read_gpx(&ulid).unwrap();
    let old_ta = io::read_track_analysis(&ulid).unwrap();

    let geojson = arrange_display(&gpx, None, None);
    
    // analyze geo data
    let ta = TrackAnalysis::new(Some(ulid), &geojson, &gpx, Some(old_ta._type));
    let geojson = arrange_display(&gpx, Some(geojson), Some(&ta.pauses));
    
    write_track_analysis(&ta).unwrap();
    write_geojson(&geojson, &ta.ulid).unwrap();
}