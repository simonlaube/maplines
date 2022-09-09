use std::{collections::HashMap, hash::Hash, ops::Add};

use gpx::Gpx;
use serde::{Deserialize, Serialize};
use geotiff::TIFF;

use crate::errors;
use crate::paths;

const SRTM_FILE_NAME: &str = "srtm_%lon_%lat.tif";

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

pub fn from_latlong(gpx: Gpx) -> Result<(), errors::MaplineError> {
    // map with key = (lon, lat), value = Tile with upper left coord (lon, lat)
    let mut tiles: HashMap<(u8, u8), TIFF> = HashMap::new();
    let mut temp_lat: i8 = 0;
    let mut temp_lon: i16 = 0;
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
            load_tile(tile_lon as u8, tile_lat as u8, &mut tiles);
        }
        let rel_lon: f64 = (w.point().x() - (temp_lon * 5) as f64) / 5.0 * 6000.;
        let rel_lat: f64 = 6000. - (w.point().y() - (temp_lat * 5) as f64) / 5.0 * 6000.;
        // println!("point lat: {}, temp lat: {}, rel_lon: {}", w.point().x(), temp_lon * 5);

        // println!("lon: {}, lat: {}", rel_lon, rel_lat);
        // println!("{}", tiles.get(&(tile_lon as u8, tile_lat as u8)).unwrap().get_value_at(rel_lat as usize, rel_lon as usize));

    }

    Err(errors::MaplineError::CouldNotLoadElevation)
}

/// Gets the tile from the srtm directory. If tile not present, tries to load
/// tile from online source.
fn load_tile(lon: u8, lat: u8, tiles: &mut HashMap<(u8, u8), TIFF>) {

    let mut lon_str: String = String::from("");
    if lon < 10 { lon_str = String::from("0"); } // add 0 in front of one digit number
    lon_str = lon_str.add(lon.to_string().as_str());

    let mut lat_str: String = String::from("");
    if lat < 10 { lat_str = String::from("0"); } // add 0 in front of one digit number

    lat_str = lat_str.add(lat.to_string().as_str());
    let mut file_name = SRTM_FILE_NAME.clone();

    let mut path = paths::srtm();
    path.push(file_name.replace("%lon", lon_str.as_str()).replace("%lat", lat_str.as_str()));
    println!("path: {:?}", path.to_str());
    let tiff = geotiff_rs::GeoTiff::from_file(path.clone());
    println!("tiff loaded");
    match TIFF::open(path.to_str().unwrap()) {
        Ok(tiff) => { tiles.insert((lon, lat), *tiff); },
        Err(e) => println!("{}", e), // TODO: load map from internet
    }
    // println!("file_name: {}", file_name.replace("%lon", lon_str.as_str()).replace("%lat", lat_str.as_str()));

}

/// Gets the elevation from tif tiles currently in memory. If coordinates lie
/// outside the tile, the corresponding tile gets added to the tile vector in memory
fn get_elevation(lat: u8, lon: u8/*, tiles: Vec< >*/) -> Result<(), errors::MaplineError> {

    Ok(())
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
}