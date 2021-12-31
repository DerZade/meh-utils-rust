use std::fs::File;
use std::io::{BufReader, Error, ErrorKind};
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct Grid {
    pub format: String,
    pub format_x: String,
    pub format_y: String,
    pub step_x: f32,
    pub step_y: f32,
    pub zoom_max: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct MetaJSON {
    pub author: String,
    pub display_name: String,
    pub elevation_offset: f32,
    pub grid_offset_x: f32,
    pub grid_offset_y: f32,
    pub grids: Vec<Grid>,
    pub latitude: f32,
    pub longitude: f32,
    pub color_outside: Option<[f32; 4]>,
    pub version: f32,
    pub world_name: String,
    pub world_size: u32,
}

pub fn from_file(path: &Path) -> Result<MetaJSON, Box<Error>> {
    if !path.is_file() {
        return Err(Box::new(Error::new(
            ErrorKind::NotFound,
            "Couldn't find meta.json",
        )));
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    match serde_json::from_reader(reader) {
        Ok(meta) => Ok(meta),
        Err(err) => Err(Box::new(Error::new(ErrorKind::Other, err.to_string()))),
    }
}
