use std::path::PathBuf;
use std::{collections::HashMap, ops::Add};

use geo::HaversineDistance;
use gpx::Gpx;
use serde::{Deserialize, Serialize};
// use geotiff::TIFF;
// use geotiff_rs::GeoTiff;
// use geotiff::TIFF;

use crate::errors::{self, MaplineError};
use crate::geotiff::TIFFStream;
use crate::{paths, pause};
use crate::io;
use crate::pause::Pause;

const SRTM_FILE_NAME: &str = "srtm_%lon_%lat.tif";
const SRTM_ZIPF_NAME: &str = "srtm_%lon_%lat.zip";
const SRTM_ADDR_NAME: &str = "https://srtm.csi.cgiar.org/wp-content/uploads/files/srtm_5x5/TIFF/srtm_%lon_%lat.zip";
const MIN_ELE_INTERVAL: f64 = 100.;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Location {
    latitude: f64,
    longitude: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OpenElevationInput {
    locations: Vec<Location>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LocationElevation {
    longitude: f64,
    elevation: i32,
    latitude: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OpenElevationResult {
    results: Vec<LocationElevation>,
}

pub fn from_latlong(gpx: &Gpx, pauses: &Vec<Pause>) -> Result<(Vec<(f64, f64)>, f64, f64, f64, f64), errors::MaplineError> {
    // map with key = (lon, lat), value = Tile with upper left coord (lon, lat)
    let mut tiles: HashMap<(u8, u8), TIFFStream> = HashMap::new();
    let mut temp_lon: i8;
    let mut temp_lat: i16;
    // let mut elevations: Vec<i32> = vec![];
    // let mut distances: Vec<f64> = vec![];
    let mut result: Vec<(f64, f64)> = vec![];
    let mut current_distance = 0.;
    let mut interval_distance = MIN_ELE_INTERVAL;

    let mut last_point = &gpx.tracks[0].segments[0].points[0].clone();
    let mut last_ele: Option<f64> = None;
    let mut current_ele: f64;

    let mut up: f64 = 0.;
    let mut down: f64 = 0.;

    let mut pause_pos = 0;

    let mut i = 0;
    for  w in &gpx.tracks[0].segments[0].points {
        // 0.00042 corresponds to half a pixel of the elevation map
        // TODO: check if this is correct (at edges of the map, on tiles with lat < 0)
        if pause_pos == pauses.len() || i < pauses.get(pause_pos).unwrap().index_before || i == pauses.get(pause_pos).unwrap().index_before {
            if w.point().y() > 0. {
                temp_lon = ((w.point().y() - 0.00042) / 5.0) as i8;
            } else {
                temp_lon = ((w.point().y() + 0.00042) / 5.0) as i8;
            }
            let tile_lat = 12 - temp_lon;
            assert!(tile_lat >= 1, "tile latitude must be greater than 1");

            let mut lon: f64 = w.point().x();
            while lon < 0. {
                lon += 360.;
            }
            temp_lat = ((w.point().x() + 0.00042) / 5.0) as i16;
            let tile_lon: i16 = ((temp_lat + 36) % 72) + 1;
            assert!(tile_lon >= 1, "tile longitude must be greater than 1");
            // println!("lat: {}, lon: {}", tile_lat, tile_lon);
            if !tiles.contains_key(&(tile_lon as u8, tile_lat as u8)) {
                load_tile(tile_lon as u8, tile_lat as u8, &mut tiles)?;
            }
            let rel_lat: f64 = (w.point().x() - (temp_lat * 5) as f64) / 5.0 * 6000.;
            let rel_lon: f64 = 6000. - (w.point().y() - (temp_lon * 5) as f64) / 5.0 * 6000.;
            // println!("point lat: {}, temp lat: {}, rel_lon: {}", w.point().x(), temp_lon * 5);

            // println!("lon: {}, lat: {}", rel_lon, rel_lat);
            // println!("{}", tiles.get(&(tile_lon as u8, tile_lat as u8)).unwrap().get_value_at(rel_lat as usize, rel_lon as usize));
            current_distance += last_point.point().haversine_distance(&w.point());
            interval_distance += last_point.point().haversine_distance(&w.point());
            last_point = w;

            // only check every nth point
            if interval_distance >= MIN_ELE_INTERVAL {
                interval_distance = 0.;
                
                let mut x_weight = rel_lon - (rel_lon as u32) as f64 - 0.5;
                let mut rel_lon_2 = rel_lon;
                // At the edge of the elevation map, elevation is not interpolated
                if x_weight < 0.0 { // use elevation to the left
                    x_weight = x_weight * -1.;
                    if rel_lon_2 as usize == 0 { break; }
                    rel_lon_2 -= 1.;
                } else { // use elevation to the right
                    if rel_lon_2 as usize == 5999 { break; }
                    rel_lon_2 += 1.;
                }
                let mut y_weight = rel_lat - (rel_lat as u32) as f64;
                let mut rel_lat_2 = rel_lat;
                if y_weight < 0.0 { // use elevation to the left
                    y_weight = y_weight * -1.;
                    if rel_lat_2 as usize == 0 { break; }
                    rel_lat_2 -= 1.;
                } else { // use elevation to the right
                    if rel_lat_2 as usize == 5999 { break; }
                    rel_lat_2 += 1.;
                }
                let tile = tiles.get(&(tile_lon as u8, tile_lat as u8)).unwrap();
                let mut ele_11 = tile.get_value_at(rel_lon as usize, rel_lat as usize) as f64;
                let mut ele_21 = tile.get_value_at(rel_lon_2 as usize, rel_lat as usize) as f64;
                
                let tile = tiles.get(&(tile_lon as u8, tile_lat as u8)).unwrap();
                let mut ele_12 = tile.get_value_at(rel_lon as usize, rel_lat_2 as usize) as f64;
                let mut ele_22 = tile.get_value_at(rel_lon_2 as usize, rel_lat_2 as usize) as f64;
                if (ele_11 - ele_21).abs() > 2000. || (ele_12 - ele_22).abs() > 2000. || (ele_11 - ele_22).abs() > 2000. {
                    ele_11 = 0.;
                    ele_21 = 0.;
                    ele_12 = 0.;
                    ele_22 = 0.;
                }

                current_ele = (ele_11 * (1. - x_weight) + ele_21 * x_weight) * (1. - y_weight) + (ele_12 * (1. - x_weight) + ele_22 * x_weight) * y_weight;
                
                // calculation without interpolation of four elevation pixels
                // current_ele = tiles.get(&(tile_lon as u8, tile_lat as u8)).unwrap().get_value_at(rel_lon as usize, rel_lat as usize) as f64;
                if current_ele > 8000. {
                    current_ele = 0.;
                }
                match last_ele {
                    Some(e) => {
                        if current_ele - e > 0. {
                            up += current_ele - e;
                        } else {
                            down += e - current_ele;
                        }
                    }
                    None => (),
                }
                last_ele = Some(current_ele);
                result.push((current_distance / 1000 as f64, current_ele));
            }
        } else if i < pauses.get(pause_pos).unwrap().index_after {
            // do nothing elevation of pause cluster not calculated
        } else {
            current_distance += last_point.point().haversine_distance(&w.point().into());
            last_point = w;
            pause_pos += 1;
        }
        i += 1;
    }
    println!("up: {}, down: {}", up, down);

    // smoothed elevation test
    let mut smoothed: Vec<(f64, f64)> = result.iter().map(|(x, y)| {
        (x.to_owned(), y.to_owned())
    }).collect();
    if result.len() < 3 {
        println!("Not enough data points for profile smoothing.");
    }
    for i in 3..result.len()-3 {
        smoothed[i].1 = (- 2. * result[i-3].1 + 3. * result[i-2].1 + 6. * result[i-1].1 + 7. * result[i].1 + 6. * result[i+1].1 + 3. * result[i+2].1 - 2. * result[i+3].1) / 21.;
        if smoothed[i].1 < 0. {
            smoothed[i].1 = 0.;
        }
    }
    let mut ele_gain = 0.;
    let mut ele_loss = 0.;
    let mut ele_max = -414.; // lowest point on earth
    let mut ele_min = 8849.; // highest point on earth

    let mut last_ele = smoothed[0].1;
    for p in &smoothed {

        if p.1 > ele_max { ele_max = p.1; }
        if p.1 < ele_min { ele_min = p.1; }

        if p.1 - last_ele > 0. {
            ele_gain += p.1 - last_ele;
        } else {
            ele_loss += last_ele - p.1;
        }
        last_ele = p.1;
    }
    println!("up_smoothed: {}, down_smoothed: {}", ele_gain, ele_loss);

    Ok((smoothed, ele_gain, ele_loss, ele_max, ele_min))

    // Err(errors::MaplineError::CouldNotLoadElevation)
}

/// Gets the tile from the srtm directory. If tile not present, tries to load
/// tile from online source.
fn load_tile(lon: u8, lat: u8, tiles: &mut HashMap<(u8, u8), TIFFStream>) -> Result<(), MaplineError> {

    let mut lon_str: String = String::from("");
    if lon < 10 { lon_str = String::from("0"); } // add 0 in front of one digit number
    lon_str = lon_str.add(lon.to_string().as_str());

    let mut lat_str: String = String::from("");
    if lat < 10 { lat_str = String::from("0"); } // add 0 in front of one digit number

    lat_str = lat_str.add(lat.to_string().as_str());
    let file_name = SRTM_FILE_NAME.clone();

    let mut path = paths::srtm();
    path.push(file_name.replace("%lon", lon_str.as_str()).replace("%lat", lat_str.as_str()));
    /*let tiff = geotiff_rs::GeoTiff::from_file(path.clone());
    match tiff {
        Ok(t) => { tiles.insert((lon, lat), t); },
        Err(e) => (),
    }*/
    /*
    match TIFF::open(path.to_str().unwrap()) {
        Ok(tiff) => { tiles.insert((lon, lat), *tiff); },
        Err(e) => println!("{}", e), // TODO: load map from internet
    }*/
    if !std::path::Path::new(&path).exists() {
        // Try downloading tiff file from
        // https://srtm.csi.cgiar.org/wp-content/uploads/files/srtm_5x5/TIFF/srtm_01_02.zip
        let address = SRTM_ADDR_NAME.replace("%lon", lon_str.as_str()).replace("%lat", lat_str.as_str());
        let mut zip_path = paths::srtm();
        zip_path.push(SRTM_ZIPF_NAME.replace("%lon", lon_str.as_str()).replace("%lat", lat_str.as_str()));
        println!("address: {}", address);
        io::download_tiff_zip(&address, PathBuf::from(&zip_path), PathBuf::from(&path));
    }
    match TIFFStream::open(path.to_str().unwrap()) {
        Ok(t) => {
            tiles.insert((lon, lat), t);
            println!("tile {} {} is opened", lon, lat);
        },
        Err(_e) => return Err(MaplineError::CouldNotLoadElevation),
    }

    println!("tiff loaded");
    Ok(())
    // println!("file_name: {}", file_name.replace("%lon", lon_str.as_str()).replace("%lat", lat_str.as_str()));

}

/*
#[tokio::main]
pub async fn from_latlong(gpx: Gpx) -> Result<(), Box<dyn std::error::Error>> {
    
    /*let mut map = HashMap::new();
    map.insert("latitude", 10.);
    map.insert("longitude", 10.);

    let mut map2 = HashMap::new();
    map2.insert("latitude", 41.161758);
    map2.insert("longitude", -8.583933);

    let mut map3 = HashMap::new();
    map3.insert("locations", [map, map2]);*/
    let mut input: OpenElevationInput = OpenElevationInput { locations: vec![] };
    let mut result: OpenElevationResult = OpenElevationResult { results: vec![] };
    let mut counter = 0;
    for w in &gpx.tracks[0].segments[0].points {
        if counter % 500 == 0 {
            match request_elevation_batch_500(&input).await {
                Ok(mut j) => {
                    result.results.append(&mut j.results);
                    input.locations = vec![]; // reset input
                }
                Err(e) => println!("{}", e),
            }
        }
        input.locations.push(Location { latitude: w.point().y(), longitude: w.point().x() });
        counter += 1;
    }

    match request_elevation_batch_500(&input).await {
        Ok(mut j) => {
            result.results.append(&mut j.results);
            input.locations = vec![]; // reset input
        }
        Err(e) => println!("{}", e),
    }

    println!("{:#?}", result);
    // println!("{:#?}", serde_json::to_string(&input).unwrap());
    // panic!();
    

    Ok(())
}
*/

/*
async fn request_elevation_batch_500(input: &OpenElevationInput) -> Result<OpenElevationResult, reqwest::Error> {
    let client = reqwest::Client::new();
    let resp = client.post("https://api.open-elevation.com/api/v1/lookup")
    .header(reqwest::header::ACCEPT, "application/json")
    .header(reqwest::header::CONTENT_TYPE, "application/json")
    //.body("{\"locations\":[{\"latitude\":7.349528574194312,\"longitude\":47.363422850781575},{\"latitude\":7.3495237126904565,\"longitude\":47.363377169409134},{\"latitude\":7.349514492596936,\"longitude\":47.363345821091166},{\"latitude\":7.349498483161824,\"longitude\":47.363327548542195},{\"latitude\":7.349503428484712,\"longitude\":47.3632995529855},{\"latitude\":7.349522371585944,\"longitude\":47.36326736647722}]}")
    /*.body("{\"locations\":[{\"latitude\": 10,
                \"longitude\": 10},{\"latitude\":20,
                \"longitude\": 20},{\"latitude\":41.161758,
                \"longitude\":-8.583933}]}")*/
    .body(serde_json::to_string(&input).unwrap())
    //.json(&input)
    .send()
    .await?;

    resp.json::<OpenElevationResult>().await
}*/