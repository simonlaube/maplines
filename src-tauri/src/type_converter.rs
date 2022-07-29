use gpx::{Gpx, Track, TrackSegment};
use geojson::{self, Feature, GeoJson, Geometry, JsonObject, JsonValue, Value, PointType};

pub fn gpx_to_geojson(gpx: &Gpx, name: &str) -> GeoJson {
    // TODO: handle multiple tracks or segments
    let track: &Track = &gpx.tracks[0];
    let segment: &TrackSegment = &track.segments[0];
    
    let mut properties = JsonObject::new();
    properties.insert(
        String::from("name"),
        JsonValue::from(name),
    );
    
    let mut line_string: Vec<PointType> = vec![];
    for w in segment.points.iter() {
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

    geojson
    /*
    let mut file = File::create(file_path).unwrap();
    write!(file, "{}", geojson.to_string()).unwrap();
    */

}