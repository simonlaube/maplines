use std::path::PathBuf;
use std::fs::File;

use std::io::Write;
use gpx::{Gpx, Track, TrackSegment, Waypoint};
use geojson::{self, Feature, GeoJson, Geometry, JsonObject, JsonValue, Value, PointType};

pub fn write_gpx_to_geojson(gpx: Gpx, file_name: String, file_path: PathBuf) {
    let track: Track = gpx.tracks[0].clone();
    let segment: TrackSegment = track.segments[0].clone();
    
    let mut properties = JsonObject::new();
    properties.insert(
        String::from("name"),
        JsonValue::from(file_name),
    );

    let mut line_string: Vec<PointType> = vec![];
    for w in segment.points {
        line_string.push(vec![w.point().x(), w.point().y()]);
    }
    let geometry = Geometry::new(Value::LineString(line_string));

    let geojson = GeoJson::Feature(Feature {
        bbox: None,
        geometry: Some(geometry),
        id: None,
        properties: Some(properties),
        foreign_members: None,
    });

    println!("{:?}", file_path);
    let mut file = File::create(file_path).unwrap();
    write!(file, "{}", geojson.to_string()).unwrap();

}