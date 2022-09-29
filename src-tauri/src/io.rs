use std::{path::PathBuf, fs::{self, File}};
use std::io::{self, BufReader, BufWriter};
use std::io::Cursor;

use geojson::GeoJson;
use gpx::{Gpx, read};
use tokio;
use serde_json;

use crate::{paths, track_analysis::TrackAnalysis};

pub fn read_geojson(ulid: &String) -> Option<GeoJson> {
    let path = paths::track_geojson(ulid);

    match fs::read_to_string(path) {
        Ok(s) => {
            match s.parse::<GeoJson>() {
                Ok(g) => Some(g),
                _ => None,
            }
        },
        _ => None,
    }
}

pub fn read_gpx(ulid: &String) -> Option<Gpx> {
    let path = paths::track_gpx(ulid);

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let gpx = read(reader); // TODO: remove unwrap
    match gpx {
        Ok(g) => Some(g),
        _ => None
    }
}

pub fn read_track_analysis(ulid: &String) -> Result<TrackAnalysis, io::Error> {
    let path = paths::track_analysis(ulid);
    let json_string = fs::read_to_string(path)?;
    let ta: TrackAnalysis = serde_json::from_str(&json_string.as_str())?;
    Ok(ta)
}

pub fn read_elevation(ulid: &str) -> Result<Vec<(f64, f64)>, io::Error> {
    let path = paths::track_elevation(ulid);
    let json_string = fs::read_to_string(path)?;
    let elevation: Vec<(f64, f64)> = serde_json::from_str(&json_string.as_str())?;
    Ok(elevation)
}

pub fn write_elevation(elevation: Vec<(f64, f64)>, ulid: &str) -> Result<(), io::Error> {
    let path = paths::track_elevation(ulid);
    write_file(path, serde_json::to_string(&elevation)?)?;
    Ok(())
}

pub fn write_track_analysis(ta: &TrackAnalysis) -> Result<(), io::Error> {
    let path = paths::track_analysis(&ta.ulid);
    write_file(path, serde_json::to_string(ta)?)?;
    Ok(())
}

pub fn write_gpx(gpx: &Gpx, ulid: &str) -> Result<(), io::Error> {
    let path = paths::track_gpx(ulid);
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    gpx::write(gpx, writer).unwrap();
    Ok(())
}

pub fn write_geojson(geojson: &GeoJson, ulid: &str) -> Result<(), io::Error> {
    let path = paths::track_geojson(ulid);
    write_file(path, geojson.to_string())?;
    Ok(())
}

fn write_file(path: PathBuf, content: String) -> Result<(), io::Error> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?
    };
    fs::write(path, content)?;
/*
    let mut file = File::create(path).unwrap();
    write!(file, "{}", content)?;
    Ok(())
    */
    Ok(())
}

// TODO: handle panics!
#[tokio::main]
pub async fn download_tiff_zip(addr: &String, zip_path: PathBuf, out_path: PathBuf) -> Result<(), reqwest::Error> {
    
    // download zip
    let response = reqwest::get(addr).await?;

    let mut zip_file = match File::create(&zip_path) {
        Err(e) => panic!("{:?}", e),
        Ok(file) => file,
    };

    let mut content = Cursor::new(response.bytes().await.unwrap());
    match std::io::copy(&mut content, &mut zip_file) {
        Ok(_o) => (),
        Err(e) => panic!("{:?}", e),
    }
    
    // unzip
    let file = fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        println!("file: {}", file.name());
        if file.name().ends_with(".tif") {
            let mut out_file = match File::create(&out_path) {
                Err(e) => panic!("{:?}", e),
                Ok(file) => file,
            };
            match std::io::copy(&mut file, &mut out_file) {
                Ok(_o) => (),
                Err(e) => panic!("{:?}", e),
            }
        }
/*
        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {} comment: {}", i, comment);
            }
        }

        if (*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }*/
    }
    Ok(())
}