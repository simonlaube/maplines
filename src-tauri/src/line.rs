use gpx::{Gpx, Track, TrackSegment};
use geojson::{self, Feature, GeoJson, Geometry, JsonObject, JsonValue, Value, PointType, LineStringType};

use crate::pause_finder::Pause;

pub fn arrange_display(gpx: &Gpx, geojson_orig: Option<GeoJson>, pauses: &Option<Vec<Pause>>) -> GeoJson {
    let geojson = match pauses {
        None => {
            match geojson_orig {
                Some(g) => g,
                None => compose(gpx, "placeholder")
            }
        }
        Some(p) => {
            compose_with_pauses(gpx, p)
        }
    };
    geojson
}

fn compose(gpx: &Gpx, name: &str) -> GeoJson {
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

fn compose_with_pauses(gpx: &Gpx, pauses: &Vec<Pause>) -> GeoJson {
    let track: &Track = &gpx.tracks[0];
    let segment: &TrackSegment = &track.segments[0];
    
    let mut properties = JsonObject::new();
    properties.insert(
        String::from("name"),
        JsonValue::from("placeholder"),
    );
    
    let mut move_lines: Vec<LineStringType> = vec![];
    let mut pause_lines: Vec<LineStringType> = vec![];
    let mut uned_pause_lines: Vec<LineStringType> = vec![];

    let mut line_move: Vec<PointType> = vec![];
    let mut line_pause: Vec<PointType> = vec![];
    let mut line_uned_pause: Vec<PointType> = vec![];
    let mut pause_pos = 0;
    for (i, w) in segment.points.iter().enumerate() {
        if pause_pos == pauses.len() {
            line_move.push(vec![w.point().x(), w.point().y()]);
        }
        // Line until pause start
        else if i < pauses.get(pause_pos).unwrap().index_before {
            line_move.push(vec![w.point().x(), w.point().y()]);
        }
        // End line and start unedited pause line
        else if i == pauses.get(pause_pos).unwrap().index_before {
            line_move.push(vec![w.point().x(), w.point().y()]);
            move_lines.push(line_move.clone());
            line_move = vec![];
            line_uned_pause.push(vec![w.point().x(), w.point().y()]);
        }
        // Unedited line until pause end
        else if i < pauses.get(pause_pos).unwrap().index_after {
            line_uned_pause.push(vec![w.point().x(), w.point().y()]);
        }
        // End unedited pause line, add direct pause line, start new line, remove first pause from vector
        else {
            line_uned_pause.push(vec![w.point().x(), w.point().y()]);
            uned_pause_lines.push(line_uned_pause.clone());
            line_uned_pause = vec![];

            line_pause.push(vec![pauses.get(pause_pos).unwrap().coord_before.0, pauses.get(pause_pos).unwrap().coord_before.1]);
            line_pause.push(vec![pauses.get(pause_pos).unwrap().coord_after.0, pauses.get(pause_pos).unwrap().coord_after.1]);
            pause_lines.push(line_pause.clone());
            line_pause = vec![];
            line_move.push(vec![w.point().x(), w.point().y()]);

            pause_pos += 1;
        }
    }
    move_lines.push(line_move);
    
    let move_geometry = Geometry::new(Value::MultiLineString(move_lines));
    let pause_geometry = Geometry::new(Value::MultiLineString(pause_lines));
    let uned_pause_geometry = Geometry::new(Value::MultiLineString(uned_pause_lines));
    let geom_coll = Geometry::new(Value::GeometryCollection(vec![move_geometry, pause_geometry, uned_pause_geometry]));

    let geojson = GeoJson::Feature(Feature {
        bbox: None,
        geometry: Some(geom_coll),
        id: None,
        properties: Some(properties),
        foreign_members: None,
    });

    geojson
}