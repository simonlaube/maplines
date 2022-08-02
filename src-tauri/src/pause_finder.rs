use std::str::FromStr;

use geo::{geometry::Coordinate, Point, EuclideanDistance, HaversineDistance};
use geojson::GeoJson;
use gpx::{Gpx, Time, Waypoint};
use chrono;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{track_analysis::TrackAnalysis, io};

/// Defines the number of points that have to consecutively lie in a cluster
const NR_CLUSTER_DETECTION_PTS: i64 = 6;
/// Min distance per second to not be considered a cluster
const MIN_ABS_DIST_SEC: f64 = 0.5;
/// relative dist / abs dist
const MIN_RATIO: f64 = 0.4;
/// Clusters within this range get declared as clusters
const SCATTER_RADIUS: f64 = 30.0;
/// Intervals greater than MAX_INTERVAL are considered pauses or teleports
const MAX_INTERVAL: i64 = 8;

/// Consecutive points must lie within a SCATTER_RADIUS for at least MIN_CLUSTER_TIME to be considered a cluster
const MIN_CLUSTER_TIME: i64 = 40;



/// Returns an Option containing the points before and after the break
/// and the time passed in seconds.
pub fn find(track_analysis: TrackAnalysis) -> Option<Vec<Pause>> {
    let min_break_time: u64 = 40;
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

    None
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
                    result.push(Pause { point_before: prev_point.unwrap().point().clone().into(), point_after: p.point().clone().into(), duration_sec: diff as u64 });
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
    while !detection_completed {

        detection_completed = true;
        let start_point = gpx.tracks[0].segments[0].points[pos].point();
        let start_time = OffsetDateTime::from(gpx.tracks[0].segments[0].points[pos].time.unwrap()).unix_timestamp();
        let mut current_cluster: Vec<&Waypoint> = vec![];
        for q in gpx.tracks[0].segments[0].points[pos..].iter() {
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
                    pos += index; // This loses time spent until index (CHECK THIS!!!)
                    detection_completed = false;
                    break;
                // check if last cluster was pause or not and continue
                } else {
                    if time_in_radius > MIN_CLUSTER_TIME {
                        let c = improve_cluster(&cluster, &center);
                        // let c = &cluster;
                        result.push(Pause { point_before: c.first().unwrap().point().into(), point_after: c.last().unwrap().point().into(), duration_sec: time_in_radius as u64 });
                        result.push(Pause { point_before: c.last().unwrap().point().into(), point_after: c.last().unwrap().point().into(), duration_sec: time_in_radius as u64 });
                        result.push(Pause { point_before: center.into(), point_after: center.into(), duration_sec: time_in_radius as u64 });
                    }
                    pos += current_cluster.len();
                    time_in_radius = 0;
                    detection_completed = false;
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

fn improve_cluster<'a>(cluster: &Vec<&'a Waypoint>, center: &'a Point<f64>) -> Vec<&'a Waypoint> {
    let mut start_center_dist: f64 = cluster[0].point().haversine_distance(center).abs();
    let mut start_index = 0;
    let init_cluster_len = cluster.len();
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
    c

}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pause {
    pub point_before: (f64, f64),
    pub point_after: (f64, f64),
    pub duration_sec: u64,
}