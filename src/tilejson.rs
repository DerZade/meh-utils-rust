use serde::Serialize;

use std::{collections::HashMap, fs::File, path::Path};
use serde_json::to_string_pretty;
use std::io::{Error, Write};
use crate::metajson::MetaJSON;

#[cfg(test)]
mod tests {
    use std::fs::{read_to_string};
    use std::num::NonZeroUsize;
    use std::ops::Add;
    use crate::metajson::MetaJSON;
    use crate::test::with_input_and_output_paths;
    use crate::tilejson::write;

    #[test]
    #[allow(unused_must_use)]
    fn tile_json_gets_written_correctly() {
        with_input_and_output_paths(|_, output_path| {
            let res = write(
                &output_path,
                5,
                MetaJSON {
                    author: "author".to_string(),
                    display_name: "display_name".to_string(),
                    elevation_offset: 0.0,
                    grid_offset_x: 1.0,
                    grid_offset_y: 2.0,
                    grids: vec![],
                    latitude: 3.0,
                    longitude: 4.0,
                    color_outside: None,
                    version: 5.0,
                    world_name: "world_name".to_string(),
                    world_size: NonZeroUsize::new(6).unwrap(),
                },
                "type_display_name",
                &Vec::new(),
                "https://localhost:3000/".to_string().add("{z}/{x}/{y}.png")
            );

            assert!(res.is_ok());

            let written = read_to_string(output_path.join("tile.json"));
            assert![written.is_ok()];
            let str = written.unwrap();

            assert!(str.contains("\"minzoom\""));
            assert!(str.contains("\"maxzoom\""));
            assert!(str.contains("\"vector_layers\""));
        });
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub struct TileJSONLayer {
    pub id: String,
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(rename = "snake_case")]
#[allow(dead_code)]
/// https://github.com/mapbox/tilejson-spec
pub struct TileJSON {
    pub tilejson: String,
    pub name: String,
    pub description: String,
    pub scheme: String,
    pub minzoom: usize,
    pub maxzoom: usize,
    pub vector_layers: Option<Vec<TileJSONLayer>>,
    pub tiles: Vec<String>
}

pub fn write(
    dir: &Path,
    max_lod: usize,
    meta: MetaJSON,
    type_display_name: &str,
    vector_layer_names: &Vec<String>,
    tile_uri: String,
) -> Result<(), Error> {
    let vector_layers: Vec<_> = vector_layer_names
        .iter()
        .map(|name| -> TileJSONLayer {
            return TileJSONLayer {
                id: name.clone(),
                fields: layer_fields(name),
            };
        })
        .collect();

    let tile_json = TileJSON {
        tilejson: String::from("2.2.0"),
        name: format!("{} {} Tiles", meta.display_name, type_display_name),
        description: format!(
            "{} Tiles of the Arma 3 Map '{}' from {}",
            type_display_name, meta.display_name, meta.author
        ),
        scheme: String::from("xyz"),
        minzoom: 0,
        maxzoom: max_lod.into(),
        vector_layers: Some(vector_layers),
        tiles: vec![tile_uri]
    };

    let mut file = File::create(dir.join("tile.json"))?;
    let json = to_string_pretty(&tile_json)?;

    file.write_all(json.as_bytes())
}

fn layer_fields(layer_name: &String) -> HashMap<String, String> {
    if layer_name == "house" {
        return [
            (
                String::from("color"),
                String::from("House color as a CSS rgb() string."),
            ),
            (
                String::from("height"),
                String::from("Bounding box height in meters"),
            ),
            (
                String::from("position"),
                String::from("Array of three floats [x, y, z]"),
            ),
        ]
        .into_iter()
        .collect();
    }

    if layer_name == "mount" {
        return [
            (
                String::from("elevation"),
                String::from("Elevation as float"),
            ),
            (
                String::from("text"),
                String::from("Rounded elevation as a string"),
            ),
        ]
        .into_iter()
        .collect();
    }

    if layer_name.starts_with("contours/") {
        return [
            (
                String::from("elevation"),
                String::from("Corrected elevation of contour. (Includes elevationOffset)"),
            ),
            (
                String::from("dem_elevation"),
                String::from("DEM elevation of contour."),
            ),
        ]
        .into_iter()
        .collect();
    }

    if layer_name.starts_with("locations/") {
        return [
            (
                String::from("name"),
                String::from("Corresponds to name value in map config."),
            ),
            (
                String::from("radiusA"),
                String::from("Corresponds to radiusA value in map config."),
            ),
            (
                String::from("radiusB"),
                String::from("Corresponds to radiusB value in map config."),
            ),
            (
                String::from("angle"),
                String::from("Corresponds to angle value in map config."),
            ),
        ]
        .into_iter()
        .collect();
    }

    return HashMap::new();
}
