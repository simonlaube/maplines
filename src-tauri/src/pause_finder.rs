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
const SCATTER_RADIUS: f64 = 50.0;

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
    /*
    for (i, p) in gpx.tracks[0].segments[0].points.iter().enumerate() {
        let mut cluster_start: geo::Point = p.point().into();
        for (j, q) in gpx.tracks[0].segments[0].points[i+1..].iter().enumerate() {
            // Find first point outside SCATTER_RADIUS
            if cluster_start.haversine_distance(&q.point().into()) > SCATTER_RADIUS {
                let diff = OffsetDateTime::from(q.time.unwrap()).unix_timestamp() - OffsetDateTime::from(p.time.unwrap()).unix_timestamp();
                // Check if time reaching this point exceeded MIN_CLUSTER_TIME
                if diff > MIN_CLUSTER_TIME {
                    println!("{}", diff);
                    break; 
                }
            }
            //println!("{}", coord_1.euclidean_distance(&Coordinate::from(q.point())));
            // coord_1.haversine_distance(&q.point().into());
            // println!("{}", coord_1.haversine_distance(&q.point().into()));
            /*if coord_1.euclidean_distance(&Coordinate::from(q.point())) < 5 {

            }*/
        }
    }
    */

    let mut result: Vec<Pause> = vec![];
    let mut detection_completed = false;
    let mut pos = 0;
    let mut time_in_radius: i64 = 0;
    let mut last_center: Point<f64> = Point::new(0.0, 0.0);
    while !detection_completed {

        detection_completed = true;
        let start_point = gpx.tracks[0].segments[0].points[pos].point();
        let start_time = OffsetDateTime::from(gpx.tracks[0].segments[0].points[pos].time.unwrap()).unix_timestamp();
        let mut cluster: Vec<&Waypoint> = vec![];
        for q in gpx.tracks[0].segments[0].points[pos..].iter() {
            if start_point.haversine_distance(&q.point().into()) < SCATTER_RADIUS || cluster.len() < 2 {
                cluster.push(q);
            } else {
                let current_time_in_radius = OffsetDateTime::from(q.time.unwrap()).unix_timestamp() - start_time;
                let (index, center) = point_closest_to_center(&cluster);
                // check if more time was spent in the current radius than in the last
                if current_time_in_radius > time_in_radius {
                    time_in_radius = current_time_in_radius;
                    last_center = center;
                    pos += index; // This loses time spent until index (CHECK THIS!!!)
                    detection_completed = false;
                    break;
                // check if last cluster was pause or not and continue
                } else {
                    if time_in_radius > MIN_CLUSTER_TIME {
                        result.push(Pause { point_before: last_center.into(), point_after: last_center.into(), duration_sec: current_time_in_radius as u64 })
                    }
                    pos += cluster.len();
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
    println!("{}", index);
    (index, center)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pause {
    pub point_before: (f64, f64),
    pub point_after: (f64, f64),
    pub duration_sec: u64,
}