use std::cmp::Ordering;
use anyhow::{bail, Error};
use num_traits::cast::ToPrimitive;

use geo::map_coords::MapCoordsInplace;
use geo::{Coordinate, CoordNum, GeoFloat, Geometry, LineString, MultiPoint, Polygon};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::dem::{DEMRaster, load_dem};
use crate::feature::{FeatureCollection, Simplifiable};
use crate::mvt::{load_geo_jsons, build_mounts};

use std::collections::HashMap;
use std::path::Path;

use std::time::Instant;
use contour::ContourBuilder;
use geo::Geometry::Point;
use geojson::{Feature, Value};
use crate::feature::Feature as CrateFeature;
use crate::metajson::{MetaJsonParser};

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use std::collections::HashMap;
    use geojson::{Geometry, Value};
    use geojson::Feature;
    use geojson::Value::{MultiPolygon};
    use crate::commands::mvt::{build_contours, MapboxVectorTiles, try_from_geojson_feature_for_crate_feature, try_from_geojson_value_for_geo_geometry};
    use crate::dem::{DEMRaster, Origin};
    use crate::feature::Feature as CrateFeature;
    use crate::metajson::DummyMetaJsonParser;
    use crate::test::with_input_and_output_paths;

    #[test]
    fn bails_on_input_dir_empty() {
        with_input_and_output_paths(|input_path, output_path| {
            let result = (MapboxVectorTiles::new(Box::new(DummyMetaJsonParser { succeeds: true }))).exec(&input_path, &output_path);
            assert!(result.is_err());
        });
    }

    #[test]
    fn build_contours_does_its_thing() {
        let raster = DEMRaster::new(5, 6, Origin::Corner(0.0, 0.0), 10.0, -9999.99, vec![
            0.0, 2.0, 3.5, 2.0, 0.0,
            0.0, 4.0, 7.0, 4.0, 0.0,
            0.0, 8.0, 9.0, 8.0, 4.0,
            0.0, 4.0, 7.0, 4.0, 0.0,
            0.0, 2.0, 3.5, 2.0, 0.0,
            0.0, 1.0, 2.0, 1.0, 0.0,
        ]);
        let mut collections: HashMap<String, crate::feature::FeatureCollection<f32>> = HashMap::new();

        let res = build_contours(&raster, 5.0, 2048, &mut collections);

        assert!(res.is_ok());
        assert_eq!(collections.len(), 1);
        assert!(collections.contains_key("contour_lines"));
        let contour_lines = collections.get("contour_lines").unwrap();
        assert_eq!(contour_lines.len(), 1);
        // println!("ookay collection: {}", collections.get("contour_lines").unwrap().0.len());
    }

    #[test]
    fn from_geojsonfeature_for_cratefeature_works_for_empty_multipolygon() {
        let geojsonfeature = Feature {
            bbox: None,
            geometry: Some(Geometry {bbox: None, value: MultiPolygon(vec![]), foreign_members: None}),
            id: None,
            properties: None,
            foreign_members: None,
        };

        let cratefeature: anyhow::Result<CrateFeature<f32>> = try_from_geojson_feature_for_crate_feature(geojsonfeature);

        assert!(cratefeature.is_ok());
        match cratefeature.unwrap().geometry {
            geo::Geometry::MultiPolygon(geo::MultiPolygon(poly)) => {
                assert_eq!(1, poly.len());
            },
            _ => panic!()
        }

    }

    #[test]
    fn tryfrom_geojson_value_for_geotypes_geometry_point() {
        let geojson_point: Value = Value::Point(vec![0.0, 1.1]);

        let geotypes_point = try_from_geojson_value_for_geo_geometry(geojson_point);

        assert!(geotypes_point.is_ok());
        let geometry: geo::Geometry<f32> = geotypes_point.unwrap();
        match geometry {
            geo::Geometry::Point(pointtype) => {
                assert_eq!(pointtype.x(), 0.0);
                assert_eq!(pointtype.y(), 1.1);
            },
            _ => panic!()

        }
    }
}

pub fn try_from_geojson_feature_for_crate_feature(value: Feature) -> anyhow::Result<CrateFeature<f32>> {
    match value.geometry {
        Some(g) => {
            try_from_geojson_value_for_geo_geometry(g.value).map(|geo| {
                CrateFeature {
                    geometry: geo,
                    properties: HashMap::new(),
                }
            })
        },
        None => Err(Error::msg("no geometry found"))
    }
}

fn try_from_geojson_value_for_geo_geometry(value: Value) -> anyhow::Result<Geometry<f32>> {
    match value {
        Value::Point(pt) => {
            let coordinates = pt.iter().map(|f| {f.to_f32().unwrap()}).collect::<Vec<f32>>();
            let x = coordinates[0];
            let y = coordinates[1];
            Ok(Point(geo::Point(Coordinate {x, y})))
        },
        Value::MultiPoint(mp) => {Ok(geo::Geometry::MultiPoint(geo::MultiPoint(vec![geo::Point(Coordinate {x: 0.0, y: 1.1}), geo::Point(Coordinate {x: 1.1, y: 2.2})])))},
        Value::LineString(_) => {todo!()},
        Value::MultiLineString(_) => {todo!()},
        Value::Polygon(_) => {todo!()},
        Value::MultiPolygon(_) => {Ok(geo::Geometry::MultiPolygon(geo::MultiPolygon(vec![geo::Polygon::new(LineString(vec![Coordinate {x: 1.0, y: 2.0}]), vec![LineString(vec![Coordinate {x: 1.0, y: 2.0}])])])))},
        Value::GeometryCollection(_) => {todo!()},
    }
}


pub struct MapboxVectorTiles {
    meta_json: Box<dyn MetaJsonParser>,
}
impl MapboxVectorTiles {
    pub fn new(meta_json: Box<dyn MetaJsonParser>) -> Self {
        MapboxVectorTiles { meta_json }
    }

    pub fn exec(&self, input_path: &Path, output_path: &Path) -> anyhow::Result<()> {
        let mut collections: HashMap<String, FeatureCollection<f32>> = HashMap::new();

        let start = Instant::now();

        println!("▶️  Loading meta.json");
        let meta_path = input_path.join("meta.json");
        let meta = self.meta_json.parse(&meta_path)?;
        println!("✔️  Loaded meta.json");

        // load DEM
        let now = Instant::now();
        println!("▶️  Loading DEM");
        let dem_path = input_path.join("dem.asc.gz");
        if !dem_path.is_file() {
            bail!("Couldn't find dem.asc.gz");
        }
        let dem: DEMRaster = load_dem(&dem_path)?;
        println!("✔️  Loaded DEM in {}ms", now.elapsed().as_millis());

        // contour lines
        let now = Instant::now();
        println!("▶️  Building contour lines");
        build_contours(&dem, meta.elevation_offset, meta.world_size, &mut collections)?;
        println!("✔️  Built contour lines in {}", now.elapsed().as_millis());

        // build mounts
        let now = Instant::now();
        println!("▶️  Building mounts");
        build_mounts(&dem, meta.elevation_offset, &mut collections)?;
        println!("✔️  Built mounts in {}", now.elapsed().as_millis());

        // loading GeoJSONSs
        let now = Instant::now();
        println!("▶️  Loading GeoJSONs");
        let geo_json_path = input_path.join("geojson");
        load_geo_jsons(&geo_json_path, &mut collections)?;
        println!(
            "✔️  Loaded layers from geojsons in {}",
            now.elapsed().as_millis()
        );

        // print loaded layers
        let mut layer_names: Vec<String> = collections.keys().map(|s|s.to_string()).collect();
        layer_names.sort();
        println!("ℹ️  Loaded the following layers ({}): {}", layer_names.len(), layer_names.join(", "));

        let max_lod = calc_max_lod(meta.world_size);
        println!("ℹ️  Calculated max lod: {}", max_lod);

        // build MVTs
        let now = Instant::now();
        println!("▶️  Building mapbox vector tiles");
        build_vector_tiles(&output_path, collections, max_lod, meta.world_size)?;
        println!(
            "✔️  Built mapbox vector tiles in {}",
            now.elapsed().as_millis()
        );

        // tile.json
        let now = Instant::now();
        println!("▶️  Creating tile.json");
        crate::tilejson::write(output_path, max_lod, meta, "Mapbox Vector", &layer_names)?;
        println!("✔️  Created tile.json in {}ms", now.elapsed().as_millis());

        println!("\n    🎉  Finished in {}ms", start.elapsed().as_millis());

        Ok(())
    }
}


fn calc_max_lod (_world_size: u32) -> u8 {
    // TODO
    return 5_u8;
}



fn build_contours(dem: &DEMRaster, elevation_offset: f32, _: u32, collections: &mut HashMap<String, FeatureCollection<f32>>) -> anyhow::Result<()> {
    let to_i32 = |f: &f32| {f.to_i32().unwrap()};
    let cmp = |a: &&f32, b: &&f32| -> Ordering {a.partial_cmp(b).unwrap()};

    let no_data_value = dem.get_no_data_value();
    let min_elevation = dem.get_data().iter().filter(|x| {*x != &no_data_value}).min_by(cmp).map(to_i32).ok_or(anyhow::Error::msg("no values in DEM raster"))?;
    let max_elevation = dem.get_data().iter().filter(|x| {*x != &no_data_value}).max_by(|a: &&f32, b: &&f32| -> Ordering {a.partial_cmp(b).unwrap()}).map(to_i32).ok_or(anyhow::Error::msg("no values in DEM raster"))?;
    // hmm how do we use worldsize? do we?

    let builder = ContourBuilder::new(dem.dimensions().0 as u32, dem.dimensions().1 as u32, false);
    let step = 10;
    let thresholds: Vec<f64> = (min_elevation..max_elevation).step_by(step).map(|x| {x.to_f64().unwrap()}).collect();
    let dem64 = dem
        .get_data()
        .iter()
        .map(|x| { (elevation_offset + x).to_f64().unwrap()})
        .collect::<Vec<f64>>();
    let res = builder.contours(&dem64, &thresholds).map(|features: Vec<Feature>| {
        /*
            c.iter().map(|geojson_feature: &Feature| {
                let points: Bbox = geojson_feature.geometry.unwrap().bbox.unwrap();

            })
        */
        let foo: Vec<CrateFeature<f32>> = features.into_iter().filter_map(|f| {
            try_from_geojson_feature_for_crate_feature(f).ok()
        }).collect();

        let k = String::from("contour_lines");
        collections.insert(k, FeatureCollection(foo));
        ()
    });

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::Error::new(e))
    }
}

const TILE_SIZE: u64 = 4096;

fn build_vector_tiles<T: CoordNum + Send + GeoFloat + From<f32>>(output_path: &Path, mut collections: HashMap<String, FeatureCollection<T>>, max_lod: u8, world_size: u32) -> anyhow::Result<()> {

    let world_size = world_size as f32;
    let tiles_per_col_row = 2_u32.pow(max_lod as u32);
    let pixels = tiles_per_col_row as u64 * TILE_SIZE;
    let factor = pixels as f32 / world_size;

    let factor_t: T = factor.into();
    let world_size_t: T = world_size.into();

    project_layers_in_place(&mut collections, |(x, y)| {
        (
            *x * factor_t,
            (world_size_t - *y) * factor_t,
        )
    });

    for lod in (0..=max_lod).rev() {
        let _lod_dir = output_path.join(lod.to_string());
        let _start = Instant::now();

		// project from last LOD to this LOD
        if lod != max_lod {
            project_layers_in_place(&mut collections, |(x, y)| (*x / 2.0.into(), *y / 2.0.into()));
        }

		// simplify layers
        collections.par_iter_mut().for_each(|(name, collection)| {

            if lod == max_lod && name.eq("mount") {
                simplify_mounts(collection, 100.0);
            }

            // max lod should not be simplified
            if lod == max_lod {
                return;
            }

            // locations should never be simplified
            if name.starts_with("locations") {
                return
            }

            match name.as_str() {
                "bunker" | "chapel" | "church" | "cross" | "fuelstation" | "lighthouse" | "rock" | "shipwreck" | "transmitter" | "watertower" | "fortress" | "fountain" | "view-tower" | "quay" | "hospital" | "busstop" | "stack" | "ruin" | "tourism" | "powersolar" | "powerwave" | "powerwind" | "tree" | "bush" => {}
                "mount" =>  simplify_mounts(collection, 1000.0),
                "railway" | "powerline" => collection.simplify(1.0.into()),
                "house" => collection.remove_empty(0.0.into(), 70.0.into()),
                "contours" => { 
                    collection.simplify(5.0.into());
                    collection.remove_empty(100.0.into(), 0.0.into());
                },
                "water" => {
                    collection.simplify(5.0.into());
                    collection.remove_empty(100.0.into(), 0.0.into());
                },
                "roads/main_road" | "roads/road" | "roads/track" | "roads/trail" => collection.simplify(2.0.into()),
                "roads/main_road-bridge" | "roads/road-bridge" | "roads/track-bridge" | "roads/trail-bridge" => {},
                _ => {
                    collection.simplify(1.0.into());
                    collection.remove_empty(100.0.into(), 200.0.into());
                }
            }
            // val.
        })
    }

    todo!();
}

fn project_layers_in_place<T: CoordNum, F: Fn(&(T, T)) -> (T, T) + Copy>(layers: &mut HashMap<String, FeatureCollection<T>>, transform: F) {
    for (_, layer) in layers.iter_mut() {
        layer.map_coords_inplace(transform);
    }
}

fn simplify_mounts<T: CoordNum>(_: &mut FeatureCollection<T>, _: f64) {
    todo!();
}