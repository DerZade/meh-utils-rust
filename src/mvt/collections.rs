use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use geo::CoordNum;
use geo::map_coords::MapCoordsInplace;
use crate::feature::FeatureCollection;
use crate::mvt::MvtGeoFloatType;

pub struct Collections(pub HashMap<String, FeatureCollection>);
impl Collections {
    pub fn new() -> Self {
        Collections(HashMap::new())
    }
}

impl MapCoordsInplace<MvtGeoFloatType> for Collections {
    fn map_coords_inplace(&mut self, func: impl Fn(&(MvtGeoFloatType, MvtGeoFloatType)) -> (MvtGeoFloatType, MvtGeoFloatType) + Copy) where MvtGeoFloatType: CoordNum {
        for (_, layer) in self.0.iter_mut() {
            layer.map_coords_inplace(func);
        }
    }
}

impl Deref for Collections {
    type Target = HashMap<String, FeatureCollection>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Collections {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}