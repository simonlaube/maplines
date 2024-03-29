use geo_types::Point;
use gpx::{Gpx, TrackSegment, Track, read, Waypoint};
use gpx::Time;
use ulid::Ulid;
use time::{OffsetDateTime};
use chrono::prelude::{DateTime, Utc};
use fitparser::{profile};

use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;

use crate::line::arrange_display;
use crate::track_analysis::{TrackAnalysis, Activity};
use crate::errors::MaplineError;
use crate::util;
use crate::io::{write_geojson, write_gpx, write_track_analysis};

pub fn gpx(gpx_path: &PathBuf) -> Result<TrackAnalysis, MaplineError> {
    let file = File::open(gpx_path).unwrap();
    let reader = BufReader::new(BufReader::new(file));
    let gpx = read(reader).unwrap(); // TODO: remove unwrap
    let geojson = arrange_display(&gpx, None, None);
    
    // TODO: implement
    optimize_gpx(&gpx);

    // TODO: take care of files with multiple tracks or segments
    // TODO: handle files with no timestamp
    let start_time = gpx.tracks[0].segments[0].points[0].time.unwrap();
    if util::track_with_start_time_exists(&start_time.format().unwrap()) {
        return Err(MaplineError::TrackAlreadyImported); // TODO: change to dialog with overrule option
    }
    
    // analyze geo data
    let track_analysis = TrackAnalysis::new(None, &geojson, &gpx, None);
    let geojson = arrange_display(&gpx, Some(geojson), Some(&track_analysis.pauses));
    write_track_analysis(&track_analysis).unwrap();
    write_geojson(&geojson, &track_analysis.ulid).unwrap();
    write_gpx(&gpx, &track_analysis.ulid).unwrap();
    
    Ok(track_analysis)
}

pub fn fit(fit_path: &PathBuf) -> Result<TrackAnalysis, MaplineError> {
    let mut fp = match File::open(fit_path) {
        Err(err) => return Err(MaplineError::ImportError(err.to_string())),
        Ok(f) => f,
    };
    // import creator and add to gpx
    let mut activity: Activity = Activity::Other;
    let mut creator: String = "unknown".to_string();
    let mut track_segment = TrackSegment::new();
    let parsed_fit = match fitparser::from_reader(&mut fp) {
        Err(err) => return Err(MaplineError::ImportError(err.to_string())),
        Ok(pf) => pf,
    };
    
    // TODO: use field enhanced_speed
    for data in parsed_fit {
        if data.kind() == profile::MesgNum::Record {
            let mut lat: Option<f64> = None;
            let mut long: Option<f64> = None;
            // let ele: i32;
            let mut timestamp: Option<DateTime<Utc>> = None;
            // println!("{:#?}", data);
            for f in data.fields() {
                match f.name() {
                    "position_lat" => lat = Some(f.value().to_string().parse::<f64>().unwrap() * 0.000000083819032),
                    "position_long" => long = Some(f.value().to_string().parse::<f64>().unwrap() * 0.000000083819032),
                    "timestamp" => timestamp = Some(f.value().to_string().parse::<DateTime<Utc>>().unwrap()),
                    // "enhanced_speed" => (),
                    _ => (),
                }
            }
            match (lat, long, timestamp) {
                (Some(la), Some(lo), Some(ti)) => {
                    let mut point = Waypoint::new(Point::new(lo, la));
                    point.time = Some(Time::from(OffsetDateTime::from_unix_timestamp(ti.timestamp()).unwrap()));
                    track_segment.points.push(point);
                }
                _ => (),
            }
        }
        
        else if data.kind() == profile::MesgNum::FileId {
            println!("{:?}", data);
            for f in data.fields() {
                match f.name() {
                    "type" => {
                        if f.value().to_string() != "activity" {
                            return Err(MaplineError::FitFileNotAnActivity);
                        }
                    }
                    "manufacturer" => creator = f.value().to_string(),
                    _ => break,
                }
            }
        }
        
        else if data.kind() == profile::MesgNum::Sport {
            // TODO: extend for more activities
            for f in data.fields() {
                match f.name() {
                    "sport" => {
                        match f.value().to_string().as_str() {
                            "xcskiing" => activity = Activity::XCSkiing, // has to be tested with xc capable device
                            "cycling" => activity = Activity::Cycling,
                            "running" => activity = Activity::Running,
                            "hiking" => activity = Activity::Hiking,
                            "walking" => activity = Activity::Hiking, // same activity as hiking
                            "swimming" => activity = Activity::Swimming, // has to be tested with swimming capable device
                            "inline_skating" => activity = Activity::InlineSkating, // has to be tested with is capable device
                            _ => activity = Activity::Other,
                        }
                    }
                    _ => (),
                }
            }
        }

    }
    let mut track = Track::new();
    track.segments.push(track_segment);
    let mut gpx = Gpx::default();
    gpx.tracks.push(track);
    gpx.version = gpx::GpxVersion::Gpx11;
    gpx.creator = Some(creator);

    let start_time = gpx.tracks[0].segments[0].points[0].time.unwrap();
    let end_time = gpx.tracks[0].segments[0].points.last().unwrap().time.unwrap();
    // TODO: check this earlier
    if util::track_with_start_time_exists(&start_time.format().unwrap()) {
        return Err(MaplineError::TrackAlreadyImported); // TODO: change to dialog with overrule option
    }
    let geojson = arrange_display(&gpx, None, None);

    let track_analysis = TrackAnalysis::new(None, &geojson, &gpx, Some(activity));
    write_track_analysis(&track_analysis).unwrap();
    let geojson = arrange_display(&gpx, Some(geojson), Some(&track_analysis.pauses));
    write_geojson(&geojson, &track_analysis.ulid).unwrap();
    write_gpx(&gpx, &track_analysis.ulid).unwrap();
    Ok(track_analysis)
}

fn optimize_gpx(_gpx: &Gpx) {
    // todo!("reduce number of track points");
}
