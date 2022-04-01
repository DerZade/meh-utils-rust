use std::collections::HashMap;
use serde::Deserialize;
use serde_json::from_str;
use crate::mvt::FeatureCollection;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use geo::Coordinate;
    use crate::feature::{Feature, FeatureCollection};
    use crate::mvt::layer_settings::find_lod_layers;

    fn some_feature() -> Feature {
        Feature {
            geometry: geo::Geometry::Point(geo::Point(Coordinate {x: 1.0, y: 1.0})),
            properties: HashMap::new(),
        }
    }

    fn collections_with_layers(layer_names: Vec<&str>) -> HashMap<String, FeatureCollection> {
        let mut collections = HashMap::new();
        layer_names.iter().for_each(|layer_name| {
            let collection = FeatureCollection::from_iter(vec![some_feature()]);

            collections.insert(layer_name.to_string(), collection);
        });

        collections
    }

    #[test]
    fn find_lod_layers_stub_returns_everything() {
        let collections = collections_with_layers(vec!["foo"]);
        let lod_layers = find_lod_layers(&collections, 1);
        assert_eq!(vec!["foo".to_string()], lod_layers);
    }

    #[test]
    fn find_lod_layers_stub_removes_contours_layer() {
        let collections = collections_with_layers(vec!["contours"]);
        let lod_layers: Vec<String> = find_lod_layers(&collections, 1);
        assert_eq!(0, lod_layers.len());
    }

    #[test]
    fn find_lod_layers_uses_default_layer_settings() {
        let collections = collections_with_layers(vec!["contours/50", "contours/100"]);
        let lod_layers: Vec<String> = find_lod_layers(&collections, 2);
        assert_eq!(vec!["contours/100".to_string()], lod_layers);

        let collections = collections_with_layers(vec!["contours/50", "contours/100"]);
        let lod_layers: Vec<String> = find_lod_layers(&collections, 3);
        assert_eq!(vec!["contours/50".to_string(), "contours/100".to_string()], lod_layers);

    }
}

#[derive(Deserialize)]
struct LayerSetting {
    pub minzoom: Option<usize>,
    pub maxzoom: Option<usize>,
    pub layer: String,
}

///
/// look into `layer_settings.json` which is formatted as follows:
/// ```ts
/// layer_settings_json = LayerSetting[];
///
/// interface LayerSetting {
///     layer: string,
///     minzoom?: number, // int>=0
///     maxzoom?: number, // int>=0
/// }
/// ```
/// *for now*, stub this.
///
/// return layer names
///
pub fn find_lod_layers(all_layers: &HashMap<String, FeatureCollection>, lod: usize) -> Vec<String> {
    let x: Vec<LayerSetting> = from_str("[{\"layer\":\"contours/100\"}, {\"layer\":\"contours/50\", \"minzoom\":3}]").unwrap();
    all_layers.keys().map(|s| {s.clone()}).filter(|k| {

        x.iter().find(|s| {
            s.layer == *k && s.minzoom.unwrap_or(0) <= lod && s.maxzoom.unwrap_or(255) >= lod
        }).is_some()
    }).collect::<Vec<String>>()
}