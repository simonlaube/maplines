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
use crate::paths;
use crate::io;

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

pub fn from_latlong(gpx: Gpx) -> Result<(Vec<(f64, i32)>, Vec<(f64, i32)>), errors::MaplineError> {
    // map with key = (lon, lat), value = Tile with upper left coord (lon, lat)
    let mut tiles: HashMap<(u8, u8), TIFFStream> = HashMap::new();
    let mut temp_lat: i8;
    let mut temp_lon: i16;
    // let mut elevations: Vec<i32> = vec![];
    // let mut distances: Vec<f64> = vec![];
    let mut result: Vec<(f64, i32)> = vec![];
    let mut current_distance = 0.;
    let mut interval_distance = MIN_ELE_INTERVAL;

    let mut last_point = &gpx.tracks[0].segments[0].points[0].clone();
    let mut last_ele: Option<f64> = None;

    let mut up: f64 = 0.;
    let mut down: f64 = 0.;
    for w in &gpx.tracks[0].segments[0].points {
        // 0.00042 corresponds to half a pixel of the elevation map
        // TODO: check if this is correct (at edges of the map, on tiles with lat < 0)
        if w.point().y() > 0. {
            temp_lat = ((w.point().y() - 0.00042) / 5.0) as i8;
        } else {
            temp_lat = ((w.point().y() + 0.00042) / 5.0) as i8;
        }
        let tile_lat = 12 - temp_lat;
        assert!(tile_lat >= 1, "tile latitude must be greater than 1");

        let mut lon: f64 = w.point().x();
        while lon < 0. {
            lon += 360.;
        }
        temp_lon = ((w.point().x() + 0.00042) / 5.0) as i16;
        let tile_lon: i16 = ((temp_lon + 36) % 72) + 1;
        assert!(tile_lon >= 1, "tile longitude must be greater than 1");
        // println!("lat: {}, lon: {}", tile_lat, tile_lon);
        if !tiles.contains_key(&(tile_lon as u8, tile_lat as u8)) {
            load_tile(tile_lon as u8, tile_lat as u8, &mut tiles)?;
        }
        let rel_lon: f64 = (w.point().x() - (temp_lon * 5) as f64) / 5.0 * 6000.;
        let rel_lat: f64 = 6000. - (w.point().y() - (temp_lat * 5) as f64) / 5.0 * 6000.;
        // println!("point lat: {}, temp lat: {}, rel_lon: {}", w.point().x(), temp_lon * 5);

        // println!("lon: {}, lat: {}", rel_lon, rel_lat);
        // println!("{}", tiles.get(&(tile_lon as u8, tile_lat as u8)).unwrap().get_value_at(rel_lat as usize, rel_lon as usize));
        current_distance += last_point.point().haversine_distance(&w.point());
        interval_distance += last_point.point().haversine_distance(&w.point());
        last_point = w;

        // only check every nth point
        if interval_distance >= MIN_ELE_INTERVAL {
            interval_distance = 0.;

            let mut ele = tiles.get(&(tile_lon as u8, tile_lat as u8)).unwrap().get_value_at(rel_lat as usize, rel_lon as usize) as i32;
            if ele > 8000 {
                ele = 0;
            }
            match last_ele {
                Some(e) => {
                    if ele as f64 - e > 0. {
                        up += ele as f64 - e;
                    } else {
                        down += e - ele as f64;
                    }
                }
                None => (),
            }
            last_ele = Some(ele as f64);
            result.push((current_distance, ele));
        }
    }
    println!("up: {}, down: {}", up, down);

    // smoothed elevation test
    let mut smoothed: Vec<(f64, i32)> = result.iter().map(|(x, y)| {
        (x.to_owned(), y.to_owned())
    }).collect();
    if result.len() < 3 {
        println!("Not enough data points for profile smoothing.");
    }
    for i in 3..result.len()-3 {
        smoothed[i].1 = (- 2 * result[i-3].1 + 3 * result[i-2].1 + 6 * result[i-1].1 + 7 * result[i].1 + 6 * result[i+1].1 + 3 * result[i+2].1 - 2 * result[i+3].1) / 21;
    }
    let mut up_smoothed = 0;
    let mut down_smoothed = 0;
    let mut last_ele = smoothed[0].1;
    for p in &smoothed {
        if p.1 - last_ele > 0 {
            up_smoothed += p.1 - last_ele;
        } else {
            down_smoothed += last_ele - p.1;
        }
        last_ele = p.1;
    }
    println!("up_smoothed: {}, down_smoothed: {}", up_smoothed, down_smoothed);

    Ok((result, smoothed))

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