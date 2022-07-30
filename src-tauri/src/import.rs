use geojson::{GeoJson};
use geo_types::Point;
use gpx::{Gpx, TrackSegment, Track, read, Waypoint};
use gpx::Time;
use ulid::Ulid;
use time::{OffsetDateTime};
use chrono::prelude::{DateTime, Utc};
use fitparser::{profile};

use std::path::PathBuf;
use std::fs::File;
use std::io::{self, Write, BufReader, BufWriter};

use crate::type_converter::gpx_to_geojson;
use crate::track_analysis::{TrackAnalysis, Activity};
use crate::paths;
use crate::errors::ImportError;

pub fn gpx(gpx_path: &PathBuf) -> Result<TrackAnalysis, io::Error> {
    let file = File::open(gpx_path)?;
    let reader = BufReader::new(file);
    let mut gpx = read(reader).unwrap(); // TODO: remove unwrap

    // TODO: implement
    optimize_gpx(&gpx);

    // TODO: take care of files with multiple tracks or segments
    let start_time = gpx.tracks[0].segments[0].points[0].time.unwrap();
    // TODO: Check if start_time already present in previous tracks
    let ulid = Ulid::from_datetime(start_time.into());
    let track: Track = gpx.tracks[0].clone();
    
    // analyze geo data
    let geojson = gpx_to_geojson(&gpx, "placeholder");
    write_geojson(&geojson, ulid.clone().to_string().as_str())?;
    write_gpx(&gpx, &ulid.to_string())?;
    
    let track_analysis = TrackAnalysis::from_import(&ulid, &start_time, &track, gpx.creator, geojson, None);
    write_track_analysis(&track_analysis)?;
    Ok(track_analysis)
}

pub fn fit(fit_path: &PathBuf) -> Result<TrackAnalysis, ImportError> {
    let mut fp = match File::open(fit_path) {
        Err(err) => return Err(ImportError::ImportError(err.to_string())),
        Ok(f) => f,
    };
    // import creator and add to gpx
    let mut activity: Activity = Activity::Generic;
    let mut track_segment = TrackSegment::new();
    let parsed_fit = match fitparser::from_reader(&mut fp) {
        Err(err) => return Err(ImportError::ImportError(err.to_string())),
        Ok(pf) => pf,
    };
    
    for data in parsed_fit {
        if data.kind() == profile::MesgNum::Record {
            let mut lat: Option<f64> = None;
            let mut long: Option<f64> = None;
            // let ele: i32;
            let mut timestamp: Option<DateTime<Utc>> = None;
            for f in data.fields() {
                match f.name() {
                    "position_lat" => lat = Some(f.value().to_string().parse::<f64>().unwrap() * 0.000000083819032),
                    "position_long" => long = Some(f.value().to_string().parse::<f64>().unwrap() * 0.000000083819032),
                    "timestamp" => timestamp = Some(f.value().to_string().parse::<DateTime<Utc>>().unwrap()),
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
            for f in data.fields() {
                match f.name() {
                    "type" => {
                        if f.value().to_string() != "activity" {
                            return Err(ImportError::FitFileNotAnActivity);
                        }
                    }
                    _ => break,
                }
            }
        }
        
        else if data.kind() == profile::MesgNum::Sport {
            // TODO: extend for more activities
            for f in data.fields() {
                match f.name() {
                    "cross_country_skiing" => activity = Activity::CrossCountrySkiing, // has to be tested with xc capable device
                    "cycling" => activity = Activity::Cycling,
                    "running" => activity = Activity::Running,
                    "hiking" => activity = Activity::Hiking,
                    "walking" => activity = Activity::Hiking, // same activity as hiking
                    "swimming" => activity = Activity::Swimming, // has to be tested with swimming capable device
                    "inline_skating" => activity = Activity::InlineSkating, // has to be tested with is capable device
                    "generic" => activity = Activity::Generic, // has to be tested
                    _ => activity = Activity::Generic,
                }
            }
        }
    }
    let mut track = Track::new();
    track.segments.push(track_segment);
    let mut gpx = Gpx::default();
    gpx.tracks.push(track);
    gpx.version = gpx::GpxVersion::Gpx11;

    let start_time = gpx.tracks[0].segments[0].points[0].time.unwrap();
    let ulid = Ulid::from_datetime(start_time.into());
    let geojson = gpx_to_geojson(&gpx, "placeholder");
    write_geojson(&geojson, ulid.clone().to_string().as_str());
    write_gpx(&gpx, &ulid.to_string());

    let track_analysis = TrackAnalysis::from_import(&ulid, &start_time, &gpx.tracks[0], gpx.creator, geojson, Some(activity));
    write_track_analysis(&track_analysis).unwrap();
    Ok(track_analysis)
}

fn optimize_gpx(gpx: &Gpx) {
    // todo!("reduce number of track points");
}

fn write_track_analysis(ta: &TrackAnalysis) -> Result<(), io::Error> {
    let mut path = paths::track_analysis();
    path.push(ta.ulid.clone().to_string());
    path.set_extension("json");
    write_file(path, serde_json::to_string(ta)?)?;
    Ok(())
}

fn write_gpx(gpx: &Gpx, ulid: &str) -> Result<(), io::Error> {
    let mut path = paths::gpx();
    path.push(ulid);
    path.set_extension("gpx");
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    gpx::write(gpx, writer).unwrap();
    Ok(())
}

fn write_geojson(geojson: &GeoJson, ulid: &str) -> Result<(), io::Error> {
    let mut path = paths::geojson();
    path.push(ulid);
    path.set_extension("geojson");
    write_file(path, geojson.to_string())?;
    Ok(())
}

fn write_file(path: PathBuf, content: String) -> Result<(), io::Error> {
    let mut file = File::create(path).unwrap();
    write!(file, "{}", content)?;
    Ok(())
}