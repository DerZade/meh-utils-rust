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

pub trait MetaJsonParser {
    fn parse(&self, path: &Path) -> Result<MetaJSON, Box<Error>>;
}
pub struct DummyMetaJsonParser {
    pub succeeds: bool
}
impl MetaJsonParser for DummyMetaJsonParser {
    fn parse(&self, _: &Path) -> Result<MetaJSON, Box<Error>> {
        if self.succeeds {
            Ok(MetaJSON {
                author: "author".to_string(),
                display_name: "display_name".to_string(),
                elevation_offset: 0.0,
                grid_offset_x: 0.0,
                grid_offset_y: 0.0,
                grids: vec![],
                latitude: 0.0,
                longitude: 0.0,
                color_outside: None,
                version: 0.1,
                world_name: "world_name".to_string(),
                world_size: 0,
            })
        } else {
            Err(Box::new(std::io::Error::new(ErrorKind::Other, "dummy error")))
        }

    }
}
pub struct SerdeMetaJsonParser {}
impl MetaJsonParser for SerdeMetaJsonParser {
    fn parse(&self, path: &Path) -> Result<MetaJSON, Box<Error>> {
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

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;
    use std::path::Path;
    use crate::metajson::MetaJsonParser;
    use crate::SerdeMetaJsonParser;

    #[test]
    fn reads_file_and_deserializes() {
        let parser = SerdeMetaJsonParser {};
        let res = parser.parse(Path::new("./resources/test/happy/input/meta.json"));

        assert!(res.is_ok());
        let meta = res.unwrap();
        assert_eq!("Bohemia Interactive", meta.author);
        assert_eq!(2048, meta.world_size);
    }

    #[test]
    fn errors_out_with_not_found_on_file_not_found() {
        let parser = SerdeMetaJsonParser {};
        let res = parser.parse(Path::new("./resources/test/happy/input/meta_not_exists.json"));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), ErrorKind::NotFound);
    }
}