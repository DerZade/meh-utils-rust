mod simplifiable;

use std::collections::HashMap;
use geo::{map_coords::MapCoordsInplace, CoordNum, Geometry};

pub use simplifiable::Simplifiable;

#[derive(Clone)]
pub enum PropertyValue {
    Null,
    Bool(bool),
    String(String),
    Number(f32),
    Array(Vec<PropertyValue>),
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

// pub type FeatureCollection<T> = Vec<Feature<T>>;

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
}
