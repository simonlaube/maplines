use geojson::{GeoJson, Feature, Value};
use geo::{self, Extremes};
use gpx::{Gpx, Track, read};
use ulid::Ulid;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use std::path::PathBuf;
use std::fs::File;
use std::io::{self, Write, BufReader, BufWriter};

use crate::{track_analysis::TrackAnalysis, type_converter::gpx_to_geojson};
use crate::paths;

pub fn gpx(gpx_path: &PathBuf) -> Result<TrackAnalysis, io::Error> {
    let file = File::open(gpx_path)?;
    let reader = BufReader::new(file);
    let mut gpx = read(reader).unwrap(); // TODO: remove unwrap

    // TODO: implement
    optimize_gpx(&gpx);

    // TODO: take care of files with multiple tracks or segments
    let start_time = gpx.tracks[0].segments[0].points[0].time.unwrap().format().unwrap();
    // TODO: Check if start_time already present in previous tracks
    let ulid = Ulid::from_datetime(OffsetDateTime::parse(start_time.as_str(), &Rfc3339).unwrap());
    let track: Track = gpx.tracks[0].clone();
    
    // analyze geo data
    let geojson = gpx_to_geojson(&gpx, "placeholder");
    write_geojson(&geojson, ulid.clone().to_string().as_str())?;
    write_gpx(&gpx, &ulid.to_string())?;
    let feature: Feature = Feature::try_from(geojson).unwrap();
    let gj_geometry: geojson::Geometry = feature.geometry.unwrap();

    // Change to Result instead of panic
    let coords = if let Value::LineString(coords) = gj_geometry.value { coords }
                                else { panic!("could not extract coords from geojson file"); };


    let geo_line: geo::LineString<f64> = Value::LineString(coords).try_into().unwrap();
    let geometry: geo::Geometry = geo_line.into();
    let extremes = geometry.extremes().unwrap();
    
    let track_analysis = TrackAnalysis {
        version: crate::ANALYSIS_VERSION,
        ulid: ulid.to_string(),
        start_time: Some(start_time),
        name: track.name,
        comment: track.comment,
        description: track.description,
        source: track.source,
        _type: track._type,
        number: track.number,
        creator: gpx.creator,
        x_min: extremes.x_min.coord.into(),
        x_max: extremes.x_max.coord.into(),
        y_min: extremes.y_min.coord.into(),
        y_max: extremes.y_max.coord.into(),
    };
    write_track_analysis(&track_analysis)?;
    Ok(track_analysis)
}

pub fn fit() {
    todo!("import .FIT files");
}

fn optimize_gpx(gpx: &Gpx) {
    // todo!("reduce number of track points");
}

fn write_track_analysis(ta: &TrackAnalysis) -> Result<(), io::Error> {
    let mut path = paths::track_analysis();
    path.push(ta.ulid.clone().to_string());
    path.set_extension("json");
    write_file(path, serde_json::to_string(ta)?);
    Ok(())
}

fn write_gpx(gpx: &Gpx, ulid: &str) -> Result<(), io::Error> {
    let mut path = paths::gpx();
    path.push(ulid);
    path.set_extension("gpx");
    println!("{:?}", &path);
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    gpx::write(gpx, writer).unwrap();
    Ok(())
}

fn write_geojson(geojson: &GeoJson, ulid: &str) -> Result<(), io::Error> {
    let mut path = paths::geojson();
    path.push(ulid);
    path.set_extension("geojson");
    write_file(path, geojson.to_string());
    Ok(())
}

fn write_file(path: PathBuf, content: String) -> Result<(), io::Error> {
    let mut file = File::create(path).unwrap();
    write!(file, "{}", content)?;
    Ok(())
}