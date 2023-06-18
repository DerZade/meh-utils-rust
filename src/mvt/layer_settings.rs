use crate::mvt::FeatureCollection;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use crate::feature::{Feature, FeatureCollection};
    use crate::mvt::layer_settings::{find_lod_layers, LayerSetting, LayerSettingSource};
    use crate::mvt::LayerSettingsFile;
    use geo::Coordinate;
    use rstest::rstest;
    use std::collections::HashMap;
    use std::path::Path;

    struct DummyLayerSettings {
        pub settings: Vec<LayerSetting>,
    }
    impl DummyLayerSettings {
        pub fn new() -> DummyLayerSettings {
            DummyLayerSettings {
                settings: vec![
                    LayerSetting {
                        layer: "always".to_string(),
                        minzoom: None,
                        maxzoom: None,
                    },
                    LayerSetting {
                        layer: "lte_lod1".to_string(),
                        minzoom: None,
                        maxzoom: Some(1),
                    },
                    LayerSetting {
                        layer: "lod3".to_string(),
                        minzoom: Some(3),
                        maxzoom: Some(3),
                    },
                    LayerSetting {
                        layer: "gte_lod3".to_string(),
                        minzoom: Some(3),
                        maxzoom: None,
                    },
                ],
            }
        }
    }
    impl LayerSettingSource for DummyLayerSettings {
        fn get_layer_settings(&self) -> anyhow::Result<Vec<LayerSetting>> {
            Ok(self.settings.clone())
        }
    }

    fn some_feature() -> Feature {
        Feature {
            geometry: geo::Geometry::Point(geo::Point(Coordinate { x: 1.0, y: 1.0 })),
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

    #[rstest]
    #[case(1, vec!["always", "lte_lod1"])]
    #[case(2, vec!["always"])]
    #[case(3, vec!["always", "gte_lod3", "lod3"])]
    #[case(4, vec!["always", "gte_lod3"])]
    fn find_lod_layers_uses_default_layer_settings(
        #[case] lod: usize,
        #[case] visible_layers: Vec<&str>,
    ) {
        let layers = vec!["lte_lod1", "always", "gte_lod3", "lod3", "never"];
        let mut visible_layers_string: Vec<String> = visible_layers
            .clone()
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let collections = collections_with_layers(layers);

        let mut lod_layers: Vec<String> =
            find_lod_layers(&collections, lod, &DummyLayerSettings::new()).unwrap();

        visible_layers_string.sort();
        lod_layers.sort();

        assert_eq!(visible_layers_string, lod_layers);
    }

    #[test]
    fn layer_settings_file_reads_file() {
        let layer_settings_res = LayerSettingsFile::from_path(
            Path::new("./resources/default_layer_settings.json").to_path_buf(),
        )
        .get_layer_settings();

        assert!(layer_settings_res.is_ok());

        let layer_settings = layer_settings_res.unwrap();

        assert_eq!(layer_settings.len(), 56);
        let first = layer_settings.first().unwrap();
        assert_eq!(
            *first,
            LayerSetting {
                layer: "debug".to_string(),
                minzoom: Some(6),
                maxzoom: None
            }
        );
        let last = layer_settings.last().unwrap();
        assert_eq!(
            *last,
            LayerSetting {
                layer: "contours/100".to_string(),
                minzoom: Some(0),
                maxzoom: Some(2)
            }
        )
    }

    #[test]
    fn layer_settings_file_errors_if_file_not_found() {
        assert!(LayerSettingsFile::from_path(
            Path::new("./resources/i_dont_exist.json").to_path_buf()
        )
        .get_layer_settings()
        .is_err());
    }

    #[test]
    fn layer_settings_file_errors_if_file_not_json() {
        assert!(LayerSettingsFile::from_path(
            Path::new("./resources/test/happy/output/.keep").to_path_buf()
        )
        .get_layer_settings()
        .is_err());
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct LayerSetting {
    pub minzoom: Option<usize>,
    pub maxzoom: Option<usize>,
    pub layer: String,
}

pub trait LayerSettingSource {
    fn get_layer_settings(&self) -> anyhow::Result<Vec<LayerSetting>>;
}

pub struct LayerSettingsFile {
    path: PathBuf,
}

impl LayerSettingsFile {
    pub fn from_path(path: PathBuf) -> LayerSettingsFile {
        LayerSettingsFile { path }
    }
}

impl LayerSettingSource for LayerSettingsFile {
    fn get_layer_settings(&self) -> anyhow::Result<Vec<LayerSetting>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).map_err(|e| anyhow::Error::new(Box::new(e)))
    }
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
pub fn find_lod_layers(
    all_layers: &HashMap<String, FeatureCollection>,
    lod: usize,
    layer_setting_source: &dyn LayerSettingSource,
) -> anyhow::Result<Vec<String>> {
    let x: Vec<LayerSetting> = layer_setting_source.get_layer_settings()?;
    Ok(all_layers
        .keys()
        .cloned()
        .filter(|k| {
            x.iter().any(|s| {
                s.layer == *k && s.minzoom.unwrap_or(0) <= lod && s.maxzoom.unwrap_or(255) >= lod
            })
        })
        .collect::<Vec<String>>())
}
