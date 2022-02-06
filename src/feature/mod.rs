mod simplifiable;

use std::collections::HashMap;
use std::rc::Rc;
use geo::{map_coords::MapCoordsInplace, map_coords::MapCoords, CoordNum, Geometry};
use mapbox_vector_tile::{Layer, Properties};
pub use simplifiable::Simplifiable;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use geo::{Coordinate, Geometry, Point};
    use mapbox_vector_tile::Layer;
    use crate::feature::{Feature, FeatureCollection};

    #[test]
    fn feature_collection_to_layer() {
        let mut feature_collection = FeatureCollection::new();
        let features =         [
            (0_usize, Coordinate {x: 1_i32, y: 1_i32}),
            (1_usize, Coordinate {x: 4_i32, y: 4_i32}),
        ];
        features.iter().for_each(|(_, c)| {
            let feature = Feature {
                geometry: geo::Geometry::Point(geo::Point(c.clone())),
                properties: HashMap::new(),
            };
            feature_collection.push(feature);
        });

        let layer: Layer = feature_collection.to_layer("foo".to_string());

        assert_eq!(2, layer.features.len());
        assert_eq!("foo".to_string(), layer.name);
        [
            (0_usize, Coordinate {x: 1_i32, y: 1_i32}),
            (1_usize, Coordinate {x: 4_i32, y: 4_i32}),
        ].into_iter().for_each(|(i, c1)| {
            if let Geometry::Point(Point(c2)) = layer.features.get(i).unwrap().geometry {
                assert_eq!(c1, c2);
            } else {
                assert!(false);
            }
        });
    }
}

#[derive(Clone)]
pub enum PropertyValue {
    Null,
    Bool(bool),
    String(String),
    Number(f32),
    Array(Vec<PropertyValue>),
}

impl Into<mapbox_vector_tile::Value> for PropertyValue {
    fn into(self) -> mapbox_vector_tile::Value {
        match self {
            PropertyValue::Null => mapbox_vector_tile::Value::Unknown,
            PropertyValue::Bool(b) => mapbox_vector_tile::Value::Boolean(b),
            PropertyValue::String(s) => mapbox_vector_tile::Value::String(Rc::new(s)),
            PropertyValue::Number(f) => mapbox_vector_tile::Value::Float(f),
            PropertyValue::Array(_) => {
                println!("WARNING array of property values will be reduced to string");
                todo!()
                // mapbox_vector_tile::Value::String(Rc::new("ARRAY OF PROPERTY VALUES".to_string()))
            }
        }
    }
}

impl From<serde_json::Value> for PropertyValue {
    fn from(val: serde_json::Value) -> Self {
        match val {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Object(_) => unimplemented!(),
            serde_json::Value::Bool(v) => Self::Bool(v),
            serde_json::Value::String(v) => Self::String(v),
            serde_json::Value::Number(v) => Self::Number(v.as_f64().unwrap() as f32),
            serde_json::Value::Array(v) => {
                let arr = v.iter().map(|e| e.into()).collect();

                Self::Array(arr)
            },
        }
    }
}

impl From<&serde_json::Value> for PropertyValue {
    fn from(val: &serde_json::Value) -> Self {
        match val {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Object(_) => unimplemented!(),
            serde_json::Value::Bool(v) => Self::Bool(v.clone()),
            serde_json::Value::String(v) => Self::String(v.clone()),
            serde_json::Value::Number(v) => Self::Number(v.as_f64().unwrap() as f32),
            serde_json::Value::Array(v) => {
                let arr = v.iter().map(|e| e.into()).collect();

                Self::Array(arr)
            },
        }
    }
}

impl PartialOrd for PropertyValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            (Self::String(a), Self::String(b)) => a.partial_cmp(b),
            _ => None
        }
    }
}

impl PartialEq for PropertyValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            _ => false
        }
    }
}

#[derive(Clone)]
pub struct Feature<T: CoordNum> {
    pub geometry: Geometry<T>,
    pub properties: HashMap<String, PropertyValue>
}

impl<T: CoordNum> Into<mapbox_vector_tile::Feature> for Feature<T> {
    fn into(self) -> mapbox_vector_tile::Feature {
        let geometry: Geometry<T> = self.geometry.clone();
        let mut new_properties: Properties = Properties::new();
        self.properties.into_iter().for_each(|(k, v)| {
            new_properties.insert(k.to_string(), v);
        });
        mapbox_vector_tile::Feature::new(
            geometry.map_coords(|(x, y)| {
                (
                    x.to_i32().unwrap(),
                    y.to_i32().unwrap(),
                )
            }),
            Rc::new(new_properties),
        )
    }
}


#[derive(Clone)]
pub struct FeatureCollection<T: CoordNum>(pub Vec<Feature<T>>);

impl<T: CoordNum> std::ops::Deref for FeatureCollection<T> {
    type Target = Vec<Feature<T>>;
    fn deref(&self) -> &Vec<Feature<T>> {
        &self.0
    }
}

impl<T: CoordNum> std::ops::DerefMut for FeatureCollection<T> {
    fn deref_mut(&mut self) -> &mut Vec<Feature<T>> {
        &mut self.0
    }
}

impl<T: CoordNum> MapCoordsInplace<T> for Feature<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T) + Copy) {
        self.geometry.map_coords_inplace(func);
    }
}

impl<T: CoordNum> FromIterator<Feature<T>> for FeatureCollection<T> {
    fn from_iter<I: IntoIterator<Item = Feature<T>>>(iter: I) -> Self {
        let mut c = Self::new();

        for i in iter {
            c.push(i);
        }

        c
    }
}

impl<T: CoordNum> MapCoordsInplace<T> for FeatureCollection<T> {
    fn map_coords_inplace(&mut self, func: impl Fn(&(T, T)) -> (T, T) + Copy) {
        for f in self.iter_mut() {
            f.map_coords_inplace(func);
        }
    }
}

impl<T: CoordNum> FeatureCollection<T> {
    pub fn new() -> Self {
        FeatureCollection(Vec::<Feature<T>>::new())
    }
    pub fn to_layer(self, name: String) -> Layer {
        let mut layer = Layer::new(name);
        self.iter().for_each(|feature| {
            layer.add_feature(feature.clone().into());
        });
        layer
    }
}
