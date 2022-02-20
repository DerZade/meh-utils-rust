use std::num::NonZeroUsize;
use geo::map_coords::MapCoordsInplace;
use num_traits::ToPrimitive;
use crate::mvt::{Collections, MvtGeoFloatType};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::num::NonZeroUsize;
    use crate::feature::{Feature, FeatureCollection};
    use crate::mvt::{ArmaMaxLodTileProjection, Collections, MvtGeoFloatType};
    use crate::mvt::project_arma_to_tile::LodProjection;

    fn assert_point_geometry(proj: &dyn LodProjection, x: &MvtGeoFloatType, y: &MvtGeoFloatType) -> () {
        let features = proj.get_collections().0.get("foo").unwrap();
        let feature: &Feature = features.0.get(0).unwrap();
        match &feature.geometry {
            geo::Geometry::Point(geo::Point(c)) => {
                assert_eq!(c, &geo::Coordinate {x: x.clone(), y: y.clone()});
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn max_projection_does_project_to_max() {
        let mut collections = Collections::new();
        let features = vec![
            Feature {
                geometry: geo::Geometry::Point(geo::Point(geo::Coordinate {x: 1.0, y: 2.0})),
                properties: HashMap::new(),
            }
        ];
        collections.0.insert("foo".to_string(), FeatureCollection(features));

        let proj = ArmaMaxLodTileProjection::new(collections, NonZeroUsize::new(1024).unwrap(), 3, 2048);

        assert_point_geometry(&proj, &16.0, &16352.0);

        assert_eq!(proj.max_lod, 3);
        assert_eq!(proj.current_lod, 3);
        assert_eq!(proj.world_size, NonZeroUsize::new(1024).unwrap());
        assert_eq!(proj.tile_size, 2048);
    }

    #[test]
    fn projection_does_decrease_lods_correctly() {

        let mut collections = Collections::new();
        let features = vec![
            Feature {
                geometry: geo::Geometry::Point(geo::Point(geo::Coordinate {x: 1.0, y: 2.0})),
                properties: HashMap::new(),
            }
        ];
        collections.0.insert("foo".to_string(), FeatureCollection(features));

        let mut proj = ArmaMaxLodTileProjection::new(collections, NonZeroUsize::new(1024).unwrap(), 3, 2048);

        assert_eq!(proj.get_lod(), 3);

        assert!(proj.decrease_lod().is_ok());

        assert_eq!(proj.get_lod(), 2);
        assert_point_geometry(&proj, &8.0, &8176.0);

        assert!(proj.decrease_lod().is_ok());

        assert_eq!(proj.get_lod(), 1);
        assert_point_geometry(&proj, &4.0, &4088.0);

        assert!(proj.decrease_lod().is_ok());

        assert_eq!(proj.get_lod(), 0);
        assert_point_geometry(&proj, &2.0, &2044.0);

        assert!(proj.decrease_lod().is_err());
    }
}

pub struct ArmaMaxLodTileProjection {
    collections: Collections,
    /// Arma3 world size: edge length in meters
    world_size: NonZeroUsize,
    /// max zoom level we want to have on the tiled map
    max_lod: usize,
    /// tile size in pixels
    tile_size: u64,
    current_lod: usize,
}
impl ArmaMaxLodTileProjection {
    pub fn new(mut collections: Collections, world_size: NonZeroUsize, max_lod: usize, tile_size: u64, ) -> Self {
        let world_size_f32 = world_size.get() as f32;
        let tiles_per_dimension = 2_u32.pow(max_lod.to_u32().unwrap());
        let pixels = tiles_per_dimension as u64 * tile_size;
        let factor = pixels as f32 / world_size_f32;
        collections.map_coords_inplace(|(x, y): &(MvtGeoFloatType, MvtGeoFloatType)| {
            (
                *x * factor,
                (world_size_f32 - *y) * factor,
            )
        });
        ArmaMaxLodTileProjection {collections, world_size, max_lod, tile_size, current_lod: max_lod.into()}
    }
}

pub trait LodProjection {
    fn get_lod(&self) -> usize;
    fn decrease_lod(&mut self) -> anyhow::Result<usize>;
    fn get_collections(&self) -> &Collections;
    fn get_collections_mut(&mut self) -> &mut Collections;
    fn is_max_lod(&self) -> bool;
}

impl LodProjection for ArmaMaxLodTileProjection {

    fn get_lod(&self) -> usize {
        self.current_lod
    }

    fn decrease_lod(&mut self) -> anyhow::Result<usize> {
        self.collections.map_coords_inplace(|&(x, y)| {
            (
                x / 2.0,
                y / 2.0
            )
        });
        self.current_lod
            .checked_sub(1)
            .map(|u| {self.current_lod = u; self.current_lod})
            .ok_or(anyhow::Error::msg("lod zero reached"))
    }

    fn get_collections(&self) -> &Collections {
        &self.collections
    }

    fn get_collections_mut(&mut self) -> &mut Collections {
        &mut self.collections
    }

    fn is_max_lod(&self) -> bool {
        self.max_lod == self.current_lod
    }
}