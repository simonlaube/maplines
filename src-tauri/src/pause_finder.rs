
use geo::{Point, HaversineDistance};
use geojson::GeoJson;
use gpx::{Gpx, Time, Waypoint};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{track_analysis::TrackAnalysis, io};

/// Scattered points within this radius can be declared as clusters -> Pauses
const SCATTER_RADIUS: f64 = 30.0;
/// Consecutive points must lie within a SCATTER_RADIUS for at least MIN_CLUSTER_TIME to be considered a cluster
/// The lower this parameter, the more clusters will be found. (Combined with a low moving speed,
/// non-existent clusters will be marked as clusters.)
/// The higher, the more clusters will be missed.
const MIN_CLUSTER_TIME: i64 = 60;
/// Intervals greater than MAX_INTERVAL are considered pauses or teleports
const MAX_INTERVAL: i64 = 8;

// Good Scatter <-> time ratio
// 30 - 60

/// Returns an Option containing the points before and after the break
/// and the time passed in seconds.
pub fn find(track_analysis: TrackAnalysis) -> Option<Vec<Pause>> {
    let geojson = io::read_geojson(&track_analysis.ulid);
    let gpx = io::read_gpx(&track_analysis.ulid);
    match (geojson, gpx) {
        (Some(gj), Some(g)) => {
            let geojson = gj;
            let gpx = g;
            return Some(find_clusters(&gpx, &geojson));
            // return find_time_jumps(&gpx, min_break_time).unwrap()
        }
        _ => return None
    }
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
                    // result.push(Pause { coord_before: prev_point.unwrap().point().clone().into(), coord_after: p.point().clone().into(), duration_sec: diff as u64 });
                }
                (Some(t), Some(p))
            },
            (None, Some(t)) => (Some(t), Some(p)),
            (_, None) => return Err("Corrupted timestamps in gpx file".to_string())
        }
    }
    Ok(Some(result))
}

/// Find consecutive gps points building 'clusters' within a constrained area
fn find_clusters(gpx: &Gpx, geojson: &GeoJson) -> Vec<Pause> {
    let mut result: Vec<Pause> = vec![];
    let mut detection_completed = false;
    let mut pos = 0;
    let mut time_in_radius: i64 = 0;
    let mut center: Point<f64> = Point::new(0.0, 0.0);
    let mut cluster: Vec<&Waypoint> = vec![];
    let mut cluster_index: usize = 0;
    while !detection_completed {

        detection_completed = true;
        let start_point = gpx.tracks[0].segments[0].points[pos].point();
        let start_time = OffsetDateTime::from(gpx.tracks[0].segments[0].points[pos].time.unwrap()).unix_timestamp();
        let mut current_cluster: Vec<&Waypoint> = vec![];
        for (i, q) in gpx.tracks[0].segments[0].points[pos..].iter().enumerate() {
            if start_point.haversine_distance(&q.point().into()) < SCATTER_RADIUS || current_cluster.len() < 2 {
                current_cluster.push(q);
            } else {
                let current_time_in_radius = OffsetDateTime::from(q.time.unwrap()).unix_timestamp() - start_time;
                let (index, current_center) = point_closest_to_center(&current_cluster);
                // check if more time was spent in the current radius than in the last
                if current_time_in_radius > time_in_radius {
                    time_in_radius = current_time_in_radius;
                    center = current_center;
                    cluster = current_cluster;
                    cluster_index = pos;
                    pos += index; // This loses time spent until index (CHECK THIS!!!)
                    detection_completed = false;
                    break;
                // check if last cluster was pause or not and continue
                } else {
                    if time_in_radius > MIN_CLUSTER_TIME {
                        match trim_cluster(&cluster, &center) {
                            Some((mut start_index, mut end_index, c)) => {
                                start_index += cluster_index;
                                end_index += cluster_index;
                                let time = OffsetDateTime::from(c.last().unwrap().time.unwrap()).unix_timestamp() - OffsetDateTime::from(c.first().unwrap().time.unwrap()).unix_timestamp();
                                result.push(Pause::new(c.first().unwrap().point().into(), start_index, c.last().unwrap().point().into(), end_index, time as u64));
                            },
                            None => (),
                        };
                    }
                    pos += current_cluster.len();
                    time_in_radius = 0;
                    detection_completed = false;
                    cluster = vec![];
                    break;
                }
            }
        }
    }
    result
}

fn point_closest_to_center(cluster: &Vec<&Waypoint>) -> (usize, Point<f64>) {
    let mut center: Point<f64> = Point::new(0.0, 0.0);
    for p in cluster {
        center += p.point();
    }
    center /= cluster.len() as f64;
    let mut shortest_dist: f64 = center.haversine_distance(&cluster.first().unwrap().point());
    let mut index: usize = 0;
    for (i, p) in cluster[1..].iter().enumerate() {
        if center.haversine_distance(&p.point()) < shortest_dist {
            shortest_dist = center.haversine_distance(&p.point());
            index = i;
        }
    }
    if index == 0 {
        index = 1;
    }
    (index, center)
}

/// Trims the in- and outgoing gps points of the cluster 
/// if they lie in a line towards the center
fn trim_cluster<'a>(cluster: &Vec<&'a Waypoint>, center: &'a Point<f64>) -> Option<(usize, usize, Vec<&'a Waypoint>)> {
    let mut start_center_dist: f64 = cluster[0].point().haversine_distance(center).abs();
    let mut start_index = 0;
    let mut c = cluster.clone();
    let mut last_time = OffsetDateTime::from(cluster[0].time.unwrap()).unix_timestamp();
    for (i, p) in cluster[1..].iter().enumerate() {
        if p.point().haversine_distance(center) < start_center_dist &&
        (OffsetDateTime::from(p.time.unwrap()).unix_timestamp() - last_time).abs() < MAX_INTERVAL {
            last_time = OffsetDateTime::from(p.time.unwrap()).unix_timestamp();
            start_center_dist = p.point().haversine_distance(center);
            start_index = i + 1;
        } else {
            break;
        }
    }
    c = c.split_at(start_index).1.into();
    c.reverse();
    last_time = OffsetDateTime::from(c[0].time.unwrap()).unix_timestamp();
    let mut end_center_dist: f64 = c[0].point().haversine_distance(center).abs();
    let mut end_index = 0;
    for (i, p) in c[1..].iter().enumerate() {
        if p.point().haversine_distance(center).abs() < end_center_dist &&
        (last_time - OffsetDateTime::from(p.time.unwrap()).unix_timestamp()).abs() < MAX_INTERVAL {
            last_time = OffsetDateTime::from(p.time.unwrap()).unix_timestamp();
            end_center_dist = p.point().haversine_distance(center);
            end_index = i + 1;
        } else {
            break;
        }
    }
    c = c.split_at(end_index).1.into();
    c.reverse();
    if OffsetDateTime::from(c.last().unwrap().time.unwrap()).unix_timestamp() - OffsetDateTime::from(c[0].time.unwrap()).unix_timestamp() < MAX_INTERVAL {
        return None
    }
    let length = c.len().clone();
    Some((start_index, start_index + length - 1, c))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pause {
    pub coord_before: (f64, f64),
    pub index_before: usize,
    pub coord_after: (f64, f64),
    pub index_after: usize,
    pub duration_sec: u64,
}

impl Pause {
    fn new(cb: (f64, f64), ib: usize, ca: (f64, f64), ia: usize, ds: u64) -> Pause {
        Pause {
            coord_before: cb,
            index_before: ib,
            coord_after: ca,
            index_after: ia,
            duration_sec: ds }
    }
}