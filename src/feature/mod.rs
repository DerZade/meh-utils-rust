mod simplifiable;

use crate::mvt::{Clip, MvtGeoFloatType};
use geo::{map_coords::MapCoords, map_coords::MapCoordsInplace, Coordinate, Geometry, Rect};
use mapbox_vector_tile::{Layer, Properties};
use num_traits::ToPrimitive;
pub use simplifiable::Simplifiable;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use crate::feature::{Feature, FeatureCollection, PropertyValue};
    use geo::{Coordinate, Geometry, Point, Rect};
    use mapbox_vector_tile::Layer;
    use num_traits::ToPrimitive;
    use rstest::rstest;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn feature_collection_to_layer() {
        let mut feature_collection = FeatureCollection::new();
        let features = [
            (0_usize, Coordinate { x: 1_i32, y: 1_i32 }),
            (1_usize, Coordinate { x: 4_i32, y: 4_i32 }),
        ];
        features.iter().for_each(|(_, c)| {
            let feature = Feature {
                geometry: geo::Geometry::Point(geo::Point(Coordinate {
                    x: c.x.to_f32().unwrap(),
                    y: c.y.to_f32().unwrap(),
                })),
                properties: HashMap::new(),
            };
            feature_collection.push(feature);
        });

        let layer: Layer = feature_collection.to_layer(
            "foo".to_string(),
            &Rect::new(
                Coordinate::zero(),
                Coordinate::from((4096.0_f32, 4096.0_f32)),
            ),
            &Coordinate::zero(),
        );

        assert_eq!(2, layer.features.len());
        assert_eq!("foo".to_string(), layer.name);
        [
            (0_usize, Coordinate { x: 1_i32, y: 1_i32 }),
            (1_usize, Coordinate { x: 4_i32, y: 4_i32 }),
        ]
        .into_iter()
        .for_each(|(i, c1)| {
            if let Geometry::Point(Point(c2)) = layer.features.get(i).unwrap().geometry {
                assert_eq!(c1, c2);
            } else {
                unreachable!()
            }
        });
    }

    #[test]
    fn PropertyValue_into_mapbox_vector_tile__Value_for_null() {
        let mvt_v: mapbox_vector_tile::Value = PropertyValue::Null.into();
        assert!(matches!(mvt_v, mapbox_vector_tile::Value::Unknown));
    }

    #[rstest]
    #[case(true)]
    #[case(false)]
    fn PropertyValue_into_mapbox_vector_tile__Value_for_bool(#[case] b: bool) {
        let mvt_v: mapbox_vector_tile::Value = PropertyValue::Bool(b).into();
        let expected = mapbox_vector_tile::Value::from(b);
        assert_eq!(mvt_v, expected);
    }

    #[test]
    fn PropertyValue_into_mapbox_vector_tile__Value_for_string() {
        let mvt_v: mapbox_vector_tile::Value = PropertyValue::String("foo".to_string()).into();
        let expected = mapbox_vector_tile::Value::String(Rc::new("foo".to_string()));
        assert_eq!(mvt_v, expected);
    }

    #[test]
    fn PropertyValue_into_mapbox_vector_tile__Value_for_number() {
        let mvt_v: mapbox_vector_tile::Value = PropertyValue::Number(333.333).into();
        let expected = mapbox_vector_tile::Value::Float(333.333);
        assert_eq!(mvt_v, expected);
    }

    #[test]
    fn PropertyValue_into_mapbox_vector_tile__Value_for_arr() {
        let arr_v = PropertyValue::Array(vec![
            PropertyValue::Null,
            PropertyValue::Bool(true),
            PropertyValue::String("foo".to_string()),
            PropertyValue::Number(42.0_f32),
            PropertyValue::Array(vec![PropertyValue::Bool(false)]),
        ]);

        let mvt_v: mapbox_vector_tile::Value = arr_v.into();

        match mvt_v {
            mapbox_vector_tile::Value::String(s) => {
                assert_eq!(s.as_str(), "[null, true, \"foo\", 42, [false]]")
            }
            _ => unreachable!("oh noez. array values should convert into string"),
        }
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

impl From<PropertyValue> for String {
    fn from(v: PropertyValue) -> String {
        match v {
            PropertyValue::Null => String::from("null"),
            PropertyValue::Bool(b) => b.to_string(),
            PropertyValue::Number(f) => f.to_string(),
            PropertyValue::String(s) => format!["\"{}\"", s],

            PropertyValue::Array(a) => {
                let strings: Vec<String> = a.into_iter().map(|v| v.into()).collect();
                format!("[{}]", strings.join(", "))
            }
        }
    }
}

impl From<PropertyValue> for mapbox_vector_tile::Value {
    fn from(val: PropertyValue) -> Self {
        match val {
            PropertyValue::Null => mapbox_vector_tile::Value::Unknown,
            PropertyValue::Bool(b) => mapbox_vector_tile::Value::Boolean(b),
            PropertyValue::String(s) => mapbox_vector_tile::Value::String(Rc::new(s)),
            PropertyValue::Number(f) => mapbox_vector_tile::Value::Float(f),
            PropertyValue::Array(arr) => {
                // println!("WARNING: array of property values will be reduced to string");
                let strings: Vec<String> = arr.into_iter().map(|v| v.into()).collect();

                mapbox_vector_tile::Value::String(Rc::new(format!("[{}]", strings.join(", "))))
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
            }
        }
    }
}

impl From<&serde_json::Value> for PropertyValue {
    fn from(val: &serde_json::Value) -> Self {
        match val {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Object(_) => unimplemented!(),
            serde_json::Value::Bool(v) => Self::Bool(*v),
            serde_json::Value::String(v) => Self::String(v.clone()),
            serde_json::Value::Number(v) => Self::Number(v.as_f64().unwrap() as f32),
            serde_json::Value::Array(v) => {
                let arr = v.iter().map(|e| e.into()).collect();

                Self::Array(arr)
            }
        }
    }
}

impl PartialOrd for PropertyValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            (Self::String(a), Self::String(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl PartialEq for PropertyValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct Feature {
    pub geometry: Geometry<MvtGeoFloatType>,
    pub properties: HashMap<String, PropertyValue>,
}
impl Feature {
    pub fn clip(&self, rect: &Rect<MvtGeoFloatType>) -> Option<Self> {
        self.geometry.clip(rect).map(|geo| crate::feature::Feature {
            geometry: geo,
            properties: self.properties.clone(),
        })
    }
    pub fn with_offset(&self, offset: &Coordinate<MvtGeoFloatType>) -> Self {
        Feature {
            geometry: self
                .geometry
                .map_coords(|(x, y)| (x - offset.x, y - offset.y)),
            properties: self.properties.clone(),
        }
    }
}

impl From<Feature> for mapbox_vector_tile::Feature {
    fn from(val: Feature) -> Self {
        let geometry: Geometry<MvtGeoFloatType> = val.geometry.clone();
        let mut new_properties: Properties = Properties::new();
        val.properties.into_iter().for_each(|(k, v)| {
            new_properties.insert(k, v);
        });
        mapbox_vector_tile::Feature::new(
            geometry.map_coords(|(x, y)| (x.to_i32().unwrap(), y.to_i32().unwrap())),
            Rc::new(new_properties),
        )
    }
}

#[derive(Clone)]
pub struct FeatureCollection(pub Vec<Feature>);
impl Deref for FeatureCollection {
    type Target = Vec<Feature>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for FeatureCollection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MapCoordsInplace<MvtGeoFloatType> for Feature {
    fn map_coords_inplace(
        &mut self,
        func: impl Fn(&(MvtGeoFloatType, MvtGeoFloatType)) -> (MvtGeoFloatType, MvtGeoFloatType) + Copy,
    ) {
        self.geometry.map_coords_inplace(func);
    }
}

impl FromIterator<Feature> for FeatureCollection {
    fn from_iter<I: IntoIterator<Item = Feature>>(iter: I) -> Self {
        let mut c = Self::new();

        for i in iter {
            c.push(i);
        }

        c
    }
}

impl MapCoordsInplace<MvtGeoFloatType> for FeatureCollection {
    fn map_coords_inplace(
        &mut self,
        func: impl Fn(&(MvtGeoFloatType, MvtGeoFloatType)) -> (MvtGeoFloatType, MvtGeoFloatType) + Copy,
    ) {
        for f in self.iter_mut() {
            f.map_coords_inplace(func);
        }
    }
}

impl FeatureCollection {
    pub fn new() -> Self {
        FeatureCollection(Vec::<Feature>::new())
    }

    pub fn to_layer(
        &self,
        name: String,
        tile_border: &Rect<MvtGeoFloatType>,
        offset: &Coordinate<MvtGeoFloatType>,
    ) -> Layer {
        let mut layer = Layer::new(name);

        self.iter()
            .filter_map(|f| f.clip(tile_border).map(|f| f.with_offset(offset).into()))
            .for_each(|f: mapbox_vector_tile::Feature| layer.add_feature(f));

        layer
    }
}
