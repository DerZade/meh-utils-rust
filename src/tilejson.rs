use serde::Serialize;

use std::{collections::HashMap, fs::File, path::Path};

use serde_json::to_string_pretty;

use std::io::{Error, Write};

use crate::metajson::MetaJSON;

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub struct TileJSONLayer {
    pub id: String,
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub struct TileJSON {
    pub tile_json: String,
    pub name: String,
    pub description: String,
    pub scheme: String,
    pub min_zoom: u8,
    pub max_zoom: u8,

    #[serde(rename = "snake_case")]
    pub vector_layers: Option<Vec<TileJSONLayer>>,
}

pub fn write(
    dir: &Path,
    max_lod: u8,
    meta: MetaJSON,
    type_display_name: &str,
    vector_layer_names: &Vec<String>,
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
        tile_json: String::from("2.2.0"),
        name: format!("{} {} Tiles", meta.display_name, type_display_name),
        description: format!(
            "{} Tiles of the Arma 3 Map '{}' from {}",
            type_display_name, meta.display_name, meta.author
        ),
        scheme: String::from("xyz"),
        min_zoom: 0,
        max_zoom: max_lod,
        vector_layers: Some(vector_layers),
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
