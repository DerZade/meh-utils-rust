use geo::map_coords::MapCoordsInplace;
use crate::mvt::{Collections, MvtGeoFloatType};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
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

        let proj = ArmaMaxLodTileProjection::new(collections, 1024, 3, 2048);

        assert_point_geometry(&proj, &16.0, &16352.0);

        assert_eq!(proj.max_lod, 3);
        assert_eq!(proj.current_lod, 3);
        assert_eq!(proj.world_size, 1024);
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

        let mut proj = ArmaMaxLodTileProjection::new(collections, 1024, 3, 2048);

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
    world_size: u32,
    /// max zoom level we want to have on the tiled map
    max_lod: u8,
    /// tile size in pixels
    tile_size: u64,
    current_lod: u8,
}
impl ArmaMaxLodTileProjection {
    pub fn new(mut collections: Collections, world_size: u32, max_lod: u8, tile_size: u64, ) -> Self {
        let world_size_f32 = world_size as f32;
        let tiles_per_dimension = 2_u32.pow(max_lod as u32);
        let pixels = tiles_per_dimension as u64 * tile_size;
        let factor = pixels as f32 / world_size_f32;
        collections.map_coords_inplace(|(x, y): &(MvtGeoFloatType, MvtGeoFloatType)| {
            (
                *x * factor,
                (world_size_f32 - *y) * factor,
            )
        });
        ArmaMaxLodTileProjection {collections, world_size, max_lod, tile_size, current_lod: max_lod}
    }
}

pub trait LodProjection {
    fn get_lod(&self) -> u8;
    fn decrease_lod(&mut self) -> anyhow::Result<()>;
    fn get_collections(&self) -> &Collections;
    fn get_collections_mut(&mut self) -> &mut Collections;
}

impl LodProjection for ArmaMaxLodTileProjection {

    fn get_lod(&self) -> u8 {
        self.current_lod
    }

    fn decrease_lod(&mut self) -> anyhow::Result<()> {
        self.collections.map_coords_inplace(|&(x, y)| {
            (
                x / 2.0,
                y / 2.0
            )
        });
        self.current_lod
            .checked_sub(1)
            .map(|u| {self.current_lod = u})
            .ok_or(anyhow::Error::msg("lod zero reached"))
    }

    fn get_collections(&self) -> &Collections {
        &self.collections
    }

    fn get_collections_mut(&mut self) -> &mut Collections {
        &mut self.collections
    }
}