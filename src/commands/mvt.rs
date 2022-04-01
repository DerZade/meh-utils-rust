use std::cmp::Ordering;
use anyhow::{bail, Error};
use num_traits::cast::ToPrimitive;

use geo::{Coordinate, Geometry, LineString, Rect};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::dem::{DEMRaster, load_dem};
use crate::feature::{FeatureCollection, Simplifiable};
use crate::mvt::{load_geo_jsons, build_mounts, find_lod_layers, MvtGeoFloatType, ArmaMaxLodTileProjection, Collections, LodProjection};

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::ops::{Add};
use std::path::{Path, PathBuf};

use std::time::Instant;
use contour::ContourBuilder;
use geo::Geometry::Point;
use geo::map_coords::MapCoordsInplace;
use geojson::{Feature, PolygonType, Value};
use mapbox_vector_tile::{Layer, Tile};
use crate::feature::Feature as CrateFeature;
use crate::metajson::{MetaJsonParser};

const DEFAULT_EXTENT: u16 = 4096;

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use std::collections::HashMap;
    use std::num::NonZeroUsize;
    use std::path::Path;
    use geo::{Coordinate};
    use geojson::{Geometry, Value};
    use geojson::Feature;
    use geojson::Value::{MultiPolygon};
    use mapbox_vector_tile::Tile;
    use rand::{Rng, thread_rng};
    use rstest::rstest;
    use crate::commands::mvt::{build_contours, build_lod_vector_tiles, build_vector_tiles, calc_max_lod, create_tile, fill_contour_layers, MapboxVectorTiles, try_from_geojson_feature_for_crate_feature, try_from_geojson_value_for_geo_geometry, vec_f64_to_coordinate_f32};
    use crate::dem::{DEMRaster, Origin};
    use crate::feature::{Feature as CrateFeature, FeatureCollection};
    use crate::metajson::DummyMetaJsonParser;
    use crate::mvt::Collections;
    use crate::test::with_input_and_output_paths;

    #[test]
    fn bails_on_input_dir_empty() {
        with_input_and_output_paths(|input_path, output_path| {
            let result = (MapboxVectorTiles::new(Box::new(DummyMetaJsonParser { succeeds: true }))).exec(&input_path, &output_path);
            assert!(result.is_err());
        });
    }

    #[test]
    fn exec_runs_successfully() {
        with_input_and_output_paths(|_, output_path| {
            let input_path = Path::new("./resources/test/happy/input").to_path_buf();
            let result =  (MapboxVectorTiles::new(Box::new(DummyMetaJsonParser { succeeds: true }))).exec(&input_path, &output_path);
            assert!(result.is_ok());
        });
    }

    #[test]
    fn build_contours_creates_empty_contour_layers_for_5_10_50_100() {
        let raster = DEMRaster::new(2, 2, Origin::Corner(0.0, 0.0), 1.0, -9999.99, vec![
            0.0, 6.0,
            1.0, 7.0,
        ]);
        let mut collections: Collections = Collections::new();
        let res = build_contours(&raster, 0.0, NonZeroUsize::new(2).unwrap(), &mut collections);

        assert!(res.is_ok());

        vec![
            "contours/05", "contours/10", "contours/50", "contours/100"
        ].into_iter().for_each(|k| {
            let layer = collections.get(k);
            assert!(layer.is_some(), "no layer {}!", k);
            assert_eq!(0, layer.unwrap().0.len());
        });
        let layer1 = collections.get("contours/01");
        assert!(layer1.is_some());
        assert_eq!(layer1.unwrap().0.len(), 8);
    }

    #[test]
    fn build_contours_does_its_thing() {
        let contour_line_to_vec_of_tuple = |feature: &CrateFeature| -> Vec<(f32, f32)> {
            match &feature.geometry {
                geo::Geometry::MultiPolygon(empty_poly) if empty_poly.0.len() == 0 => vec![],
                geo::Geometry::MultiPolygon(some_poly) => {
                    let poly = some_poly.0.get(0).unwrap();
                    let ext = poly.exterior();
                    ext.0.iter().map(|f| { (f.x.clone(), f.y.clone()) }).collect()
                }
                _ => vec![],
            }
        };

        let raster = DEMRaster::new(5, 6, Origin::Corner(0.0, 0.0), 10.0, -9999.99, vec![
            0.0, 0.5, 0.5, 0.0, 0.0,
            1.0, 3.0, 3.0, 1.0, 0.0,
            1.0, 7.0, 5.0, 3.0, 1.0,
            1.0, 9.0, 5.0, 5.0, 1.0,
            1.0, 7.0, 5.0, 3.0, 0.0,
            0.0, 1.0, 1.0, 1.0, 0.0,
        ]);
        let mut collections: Collections = Collections::new();

        let res = build_contours(&raster, 50.0, NonZeroUsize::new(2048).unwrap(), &mut collections);

        assert!(res.is_ok());
        assert!(collections.contains_key("contours/01"));
        let contour_lines: &FeatureCollection = collections.get("contours/01").unwrap();
        assert_eq!(contour_lines.len(), 10);
        println!("ookay collection: {}", collections.get("contours/01").unwrap().0.len());

        let v = contour_line_to_vec_of_tuple(contour_lines.0.get(0).unwrap());

        assert_eq!(v, vec![(50.0, 1993.0), (50.0, 2003.0), (50.0, 2013.0), (50.0, 2023.0), (50.0, 2033.0), (50.0, 2043.0), (45.0, 2048.0), (35.0, 2048.0), (25.0, 2048.0), (15.0, 2048.0), (5.0, 2048.0), (0.0, 2043.0), (0.0, 2033.0), (0.0, 2023.0), (0.0, 2013.0), (0.0, 2003.0), (0.0, 1993.0), (5.0, 1988.0), (15.0, 1988.0), (25.0, 1988.0), (35.0, 1988.0), (45.0, 1988.0), (50.0, 1993.0)]);

        let v = contour_line_to_vec_of_tuple(contour_lines.0.get(2).unwrap());

        assert_eq!(v, vec![(40.0, 2003.0), (40.0, 2013.0), (40.0, 2023.0), (35.0, 2028.0), (30.0, 2033.0), (25.0, 2038.0), (15.0, 2038.0), (10.0, 2033.0), (10.0, 2023.0), (10.0, 2013.0), (10.0, 2003.0), (15.0, 1998.0), (25.0, 1998.0), (35.0, 1998.0), (40.0, 2003.0)]);

        let v = contour_line_to_vec_of_tuple(contour_lines.0.get(4).unwrap());

        assert_eq!(v, vec![(30.0, 2003.0), (35.0, 2008.0), (40.0, 2013.0), (35.0, 2018.0), (30.0, 2023.0), (25.0, 2028.0), (15.0, 2028.0), (10.0, 2023.0), (10.0, 2013.0), (10.0, 2003.0), (15.0, 1998.0), (25.0, 1998.0), (30.0, 2003.0)]);

        let v = contour_line_to_vec_of_tuple(contour_lines.0.get(6).unwrap());

        assert_eq!(v, vec![(20.0, 2003.0), (20.0, 2013.0), (20.0, 2023.0), (15.0, 2028.0), (10.0, 2023.0), (10.0, 2013.0), (10.0, 2003.0), (15.0, 1998.0), (20.0, 2003.0)]);

        let v = contour_line_to_vec_of_tuple(contour_lines.0.get(8).unwrap());

        assert_eq!(v, vec![(20.0, 2013.0), (15.0, 2018.0), (10.0, 2013.0), (15.0, 2008.0), (20.0, 2013.0)]);
    }

    #[test]
    fn vec_f64_to_coordinate_f32_all_the_things() {
        assert_eq!(Coordinate {x: 0.0, y: 1.1 }, vec_f64_to_coordinate_f32(&vec![0.0_f64, 1.1_f64]).unwrap());
        assert!(vec_f64_to_coordinate_f32(&vec![0.0]).is_err());
    }

    #[test]
    fn from_geojsonfeature_for_cratefeature_works_for_empty_multipolygon() {
        let geojsonfeature = Feature {
            bbox: None,
            geometry: Some(Geometry {bbox: None, value: MultiPolygon(vec![ // one multipolygon consists of n polygons
                vec![ // one polygon with one or more linear rings
                    vec![ // one linear ring with n points, denoting the polygon surface
                        vec![0.0, 1.1],
                        vec![1.1, 2.2],
                        vec![2.2, 0.0],
                    ],
                      // …optionally, more linear rings for holes in the surface
                ]
            ]), foreign_members: None}),
            id: None,
            properties: None,
            foreign_members: None,
        };

        let cratefeature: anyhow::Result<CrateFeature> = try_from_geojson_feature_for_crate_feature(geojsonfeature);

        assert!(cratefeature.is_ok());
        match cratefeature.unwrap().geometry {
            geo::Geometry::MultiPolygon(geo::MultiPolygon(poly)) => {
                assert_eq!(1, poly.len());
                let first_poly = &poly[0];
                let exterior = &first_poly.exterior().0;
                assert_eq!(exterior.first().unwrap(), &Coordinate {x: 0.0_f32, y: 1.1_f32 });
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

    #[test]
    fn build_vector_tiles_does_not_explode_on_empty_input() {
        with_input_and_output_paths(|_, output_path| {
            let res = build_vector_tiles(&output_path, Collections::new(), 1, NonZeroUsize::new(1).unwrap());

            assert!(res.is_ok());
        });
    }

    fn some_feature() -> CrateFeature {
        let mut rng = thread_rng();
        let mut rand = || {rng.gen_range(0.0..127.0)};
        CrateFeature {
            geometry: geo::Geometry::Point(geo::Point(Coordinate {x:  rand(), y: rand()})),
            properties: HashMap::new(),
        }
    }

    fn collections_with_layers(layer_names: Vec<&str>/*, add_features: bool*/) -> HashMap<String, FeatureCollection> {
        let mut collections = HashMap::new();
        layer_names.iter().for_each(|layer_name| {
            let collection = FeatureCollection::from_iter(vec![]);
            collections.insert(layer_name.to_string(), collection);
        });

        collections
    }

    #[test]
    fn fill_contour_layers_does_not_panic_if_no_contours_but_returns_err() {
        let mut layers = HashMap::<String, FeatureCollection>::new();
        let res = fill_contour_layers(vec!["foo".to_string(), "contours/1".to_string()], &mut layers);

        assert!(res.is_err());
    }

    #[test]
    fn fill_contour_layers_copies_all_features_from_contours_to_contours_1() {
        let contours_layer_name = "contours";
        let mut layers = collections_with_layers(vec![contours_layer_name, "contours/1", "foo"]);
        layers.get_mut("foo").unwrap().push(some_feature());
        layers.get_mut(contours_layer_name).unwrap().push(some_feature());
        layers.get_mut(contours_layer_name).unwrap().push(some_feature());

        fill_contour_layers(layers.keys().map(|f| {f.to_string()}).collect(), &mut layers);

        let contours_1_features = &layers.get("contours/1").unwrap().0;
        let contours_features = &layers.get(contours_layer_name).unwrap().0;
        assert_eq!(2, contours_1_features.len());
        for i in 0..=1 {
            assert_eq!(contours_features.get(i).unwrap().geometry, contours_1_features.get(i).unwrap().geometry);
        }
        assert_eq!(1, layers.get("foo").unwrap().len());
    }

    #[test]
    fn fill_contour_layers_copies_only_every_fifth_feature_from_contours_to_contours_5() {
        let mut layers = collections_with_layers(vec!["contours", "contours/05", "foo"]);
        for _ in 0..11 {
            layers.get_mut("contours").unwrap().push(some_feature());
        }

        fill_contour_layers(layers.keys().map(|f| {f.to_string()}).collect(), &mut layers);

        let contours_5_features = &layers.get("contours/05").unwrap().0;
        let contours_features = &layers.get("contours").unwrap().0;
        assert_eq!(3, contours_5_features.len());
        assert_eq!(contours_features.get(0).unwrap().geometry, contours_5_features.get(0).unwrap().geometry);
        assert_eq!(contours_features.get(5).unwrap().geometry, contours_5_features.get(1).unwrap().geometry);
        assert_eq!(contours_features.get(10).unwrap().geometry, contours_5_features.get(2).unwrap().geometry);
    }

    #[test]
    fn build_lod_vector_tiles_creates_directories() {
        with_input_and_output_paths(|_, output_path| {
            let mut layers = collections_with_layers(vec!["bar"]);

            let lod_path = output_path.join("11");
            let res = build_lod_vector_tiles(&mut layers, NonZeroUsize::new(4096).unwrap(), 2, &lod_path);

            assert!(res.is_ok());
            assert!(output_path.is_dir());
            assert!(lod_path.is_dir());

            assert!(lod_path.join("0").is_dir());
            assert!(lod_path.join("3").is_dir());
            assert!(!lod_path.join("4").is_dir());
        });
    }

    #[test]
    fn create_tile_returns_empty_tile_if_no_layers() {
        let mut collections: HashMap<String, FeatureCollection> = HashMap::new();
        let tile_res = create_tile(0, 0, &mut collections);

        assert!(tile_res.is_ok());
        let tile = tile_res.unwrap();

        assert!(tile.is_empty());
        assert_eq!(0, tile.layers.len());
    }

    #[test]
    fn create_tile_returns_empty_tile_if_one_empty_layer() {
        let mut collections: HashMap<String, FeatureCollection> = HashMap::new();
        collections.insert("foo".to_string(), FeatureCollection(vec![]));

        let tile_res = create_tile(0, 0, &mut collections);
        assert!(tile_res.is_ok());
        let tile = tile_res.unwrap();
        assert_eq!(1, tile.layers.len());
        let empty_layer = tile.layers.get(0).unwrap();
        assert!(empty_layer.features.is_empty());
        assert_eq!("foo".to_string(), empty_layer.name);
        assert_eq!(4096, empty_layer.extent);
    }


    #[test]
    fn create_tile_returns_tile_with_features() {

        fn tile_has_point_at(tile: &Tile, x: i32, y: i32) {
            assert!(!tile.is_empty());
            let maybe_layer = tile.layers.get(0);
            assert!(maybe_layer.is_some());
            let layer = maybe_layer.unwrap();
            assert_eq!(1, layer.features.len());
            let feat = layer.features.get(0).unwrap();
            match feat.geometry {
                geo::Geometry::Point(geo::Point(c)) => {
                    assert_eq!(c.x, x);
                    assert_eq!(c.y, y)
                },
                _ => assert!(false)
            }
        }


        let mut collections: HashMap<String, FeatureCollection> = HashMap::new();
        let mut foo_features = FeatureCollection(vec![]);
        let point_on_tile = geo::Geometry::Point(geo::Point(Coordinate {x: 1.0, y: 1.0}));
        foo_features.push(CrateFeature {geometry: point_on_tile, properties: HashMap::new()});
        let point_off_tile = geo::Geometry::Point(geo::Point(Coordinate {x: 5000.0, y: 5000.0}));
        foo_features.push(CrateFeature {geometry: point_off_tile, properties: HashMap::new()});

        collections.insert("foo".to_string(), foo_features);

        let tile_res = create_tile(0, 0, &mut collections);

        assert!(tile_res.is_ok());

        tile_has_point_at(&tile_res.unwrap(), 1, 1);

        let tile_res = create_tile(1, 1, &mut collections);

        assert!(tile_res.is_ok());

        tile_has_point_at(&tile_res.unwrap(), 5000 - 4096, 5000 - 4096);
    }

    #[rstest]
    #[case(256, 4096, 0)]
    #[case(512, 4096, 1)]
    #[case(1024, 4096, 2)]
    #[case(2048, 4096, 3)]
    fn calc_max_lod_returns_good_values(#[case] world_size: usize, #[case] tile_size: usize, #[case] expected_lod: usize) {
        let res_lod = calc_max_lod(NonZeroUsize::new(world_size).unwrap(), tile_size);
        assert!(res_lod.is_ok());
        assert_eq!(res_lod.unwrap(), expected_lod);
    }
}

pub fn try_from_geojson_feature_for_crate_feature(value: Feature) -> anyhow::Result<CrateFeature> {
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

fn vec_f64_to_coordinate_f32(point: &Vec<f64>) -> anyhow::Result<Coordinate<f32>> {
    if point.len() < 2 {
        Err(anyhow::Error::msg("vector is no coordinate pair: less than 2 values"))
    } else {
        match (point.get(0).map(|x| {x.to_f32()}), point.get(1).map(|x| {x.to_f32()})) {
            (Some(Some(x)), Some(Some(y))) => Ok(Coordinate::from((x,y))),
            _ => Err(anyhow::Error::msg("cannot convert vector to f32"))
        }
    }
}

fn try_from_geojson_value_for_geo_geometry(value: Value) -> anyhow::Result<Geometry<f32>> {
    match value {
        Value::Point(pt) => {
            vec_f64_to_coordinate_f32(&pt).map(|c| {
                Point(geo::Point(c))
            })
        },
        Value::MultiPoint(_) => {Ok(geo::Geometry::MultiPoint(geo::MultiPoint(vec![geo::Point(Coordinate {x: 0.0, y: 1.1}), geo::Point(Coordinate {x: 1.1, y: 2.2})])))},
        Value::LineString(_) => {todo!()},
        Value::MultiLineString(_) => {todo!()},
        Value::Polygon(_) => {todo!()},
        Value::MultiPolygon(mp) => {
            let poly_results: anyhow::Result<Vec<geo::Polygon<f32>>> = mp.iter().map(|poly: &PolygonType| {
                // poly consists of linestrings:
                let linestring_results: anyhow::Result<Vec<LineString<f32>>> = poly.iter().map(|line| {
                    let res: anyhow::Result<Vec<Coordinate<f32>>> = line.iter().map(vec_f64_to_coordinate_f32).into_iter().collect();
                    res.map(|cs| {
                        LineString(cs)
                    })
                }).collect();
                linestring_results.map(|linestrings| {
                    geo::Polygon::new(
                        linestrings.first().unwrap().clone(),
                        vec![]
                    )
                })
            }).collect();
            poly_results.map(|p| {geo::Geometry::MultiPolygon(geo::MultiPolygon(p))})
        },
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
        let mut collections: Collections = Collections::new();

        let start = Instant::now();

        let now = Instant::now();
        println!("▶️  Loading meta.json");
        let meta_path = input_path.join("meta.json");
        let meta = self.meta_json.parse(&meta_path)?;
        println!("✔️  Loaded meta.json in {}μs", now.elapsed().as_micros());

        // load DEM
        let now = Instant::now();
        println!("▶️  Loading DEM");
        let dem_path = input_path.join("dem.asc.gz");
        if !dem_path.is_file() {
            bail!("Couldn't find dem.asc.gz");
        }
        let dem: DEMRaster = load_dem(&dem_path)?;
        println!("✔️  Loaded DEM in {}μs", now.elapsed().as_micros());

        // contour lines
        let now = Instant::now();
        println!("▶️  Building contour lines");
        build_contours(&dem, meta.elevation_offset, meta.world_size, &mut collections)?;
        println!("✔️  Built contour lines in {}μs", now.elapsed().as_micros());

        // build mounts
        let now = Instant::now();
        println!("▶️  Building mounts");
        build_mounts(&dem, meta.elevation_offset, &mut collections)?;
        println!("✔️  Built mounts in {}μs", now.elapsed().as_micros());

        // loading GeoJSONSs
        let now = Instant::now();
        println!("▶️  Loading GeoJSONs");
        let geo_json_path = input_path.join("geojson");
        load_geo_jsons(&geo_json_path, &mut collections)?;
        println!(
            "✔️  Loaded layers from geojsons in {}μs",
            now.elapsed().as_micros()
        );

        // print loaded layers
        let mut layer_names: Vec<String> = collections.keys().map(|s|s.to_string()).collect();
        layer_names.sort();
        println!("ℹ️  Loaded the following layers ({}): {}", layer_names.len(), layer_names.join(", "));

        let max_lod = calc_max_lod(meta.world_size, DEFAULT_EXTENT.to_usize().unwrap())?;
        println!("ℹ️  Calculated max lod: {}", max_lod);

        // build MVTs
        let now = Instant::now();
        println!("▶️  Building mapbox vector tiles");
        build_vector_tiles(&output_path, collections, max_lod, meta.world_size)?;
        println!(
            "✔️  Built mapbox vector tiles in {}μs",
            now.elapsed().as_micros()
        );

        // tile.json
        let now = Instant::now();
        println!("▶️  Creating tile.json");
        crate::tilejson::write(output_path, max_lod, meta, "Mapbox Vector", &layer_names, "https://localhost/".to_string().add("{z}/{x}/{y}.pbf"))?;
        println!("✔️  Created tile.json in {}μs", now.elapsed().as_micros());

        println!("\n    🎉  Finished in {}μs", start.elapsed().as_micros());

        Ok(())
    }
}

fn calc_max_lod(world_size: NonZeroUsize, tile_size: usize) -> anyhow::Result<usize> {
    // lets say we want a resolution of 10cm, that is 10_000px/km. tiles are 4096px, so going from world_size that would be
    let tile_size_f64 = tile_size.to_f64().ok_or(anyhow::Error::msg(format!("could not convert {} to f64", tile_size)))?;
    let world_size_f64 = (world_size).get().to_f64().ok_or(anyhow::Error::msg(format!("could not convert {} to f64", world_size)))?;
    (world_size_f64 * 10.0_f64 / tile_size_f64).max(1.0_f64).log2().ceil().to_usize().ok_or(anyhow::Error::msg("could not convert to usize. Negative value?"))
}

fn build_contours(dem: &DEMRaster, elevation_offset: f32, world_size: NonZeroUsize, collections: &mut Collections) -> anyhow::Result<()> {
    let cmp = |a: &&f64, b: &&f64| -> Ordering {a.partial_cmp(b).unwrap()};
    let cell_size = dem.get_cell_size();

    let expand_by_cell_size = |(a, b): &(f32, f32)| -> (f32, f32) {(a * cell_size, world_size.get().to_f32().unwrap() - (b * cell_size))};

    let no_data_value: f64 = dem.get_no_data_value().to_f64().unwrap();
    let dem64 = dem
        .get_data()
        .into_iter()
        .map(|i| {i.to_f64().unwrap()})
        .collect::<Vec<f64>>();


    let min_elevation = dem64.iter().filter(|x| {*x != &no_data_value}).min_by(cmp).ok_or(anyhow::Error::msg("no values in DEM raster"))?;
    let max_elevation = dem64.iter().filter(|x| {*x != &no_data_value}).max_by(cmp).ok_or(anyhow::Error::msg("no values in DEM raster"))?;
    // hmm how do we use worldsize? do we?

    let builder = ContourBuilder::new(dem.dimensions().0 as u32, dem.dimensions().1 as u32, false);
    let thresholds: Vec<f64> = (min_elevation.to_i64().unwrap() ..=max_elevation.to_i64().unwrap()).map(|f| {f.to_f64().unwrap()}).collect();

    println!("contour builder: dimensions {} by {}", dem.dimensions().0 as u32, dem.dimensions().1 as u32);
    println!("contour builder: elevation min {}, max {}, offset {}",  min_elevation, max_elevation, elevation_offset);

    let res = builder.contours(&dem64, &thresholds).map(|features: Vec<Feature>| {
        let mut foo: Vec<CrateFeature> = features.into_iter().filter_map(|f| {
            try_from_geojson_feature_for_crate_feature(f).ok()
        }).collect();

        foo.iter_mut().for_each(|feature| {
            feature.map_coords_inplace(expand_by_cell_size);
        });

        collections.insert(String::from("contours/01"), FeatureCollection(foo));
        ()
    });

    // define *empty* 5,10,50,100 contour line layers, to be filled *later* after lod-specific selection!
    collections.insert("contours/05".to_string(), FeatureCollection(vec![]));
    collections.insert("contours/10".to_string(), FeatureCollection(vec![]));
    collections.insert("contours/50".to_string(), FeatureCollection(vec![]));
    collections.insert("contours/100".to_string(), FeatureCollection(vec![]));

    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::Error::new(e))
    }
}

const TILE_SIZE: u64 = 4096;

fn build_vector_tiles(output_path: &Path, collections: Collections, max_lod: usize, world_size: NonZeroUsize) -> anyhow::Result<()> {
    let mut projection = ArmaMaxLodTileProjection::new(collections, world_size, max_lod, TILE_SIZE);

    let mut projection_lod = Ok(max_lod);
    while let Ok(lod) = projection_lod {
        let now = Instant::now();

        let lod_dir: PathBuf = output_path.join(lod.to_string());

		// simplify layers
        let is_max_lod = projection.is_max_lod();
        projection.get_collections_mut().par_iter_mut().for_each(|(name, collection)| {

            if is_max_lod && name.eq("mount") {
                simplify_mounts(collection, 100.0);
            }

            // max lod should not be simplified
            if is_max_lod {
                return;
            }

            // locations should never be simplified
            if name.starts_with("locations") {
                return
            }
            println!("simplifying {} at lod {}", name.as_str(), lod);

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
        });


        let lod_layer_names = find_lod_layers(projection.get_collections_mut(), lod);
        // note: the following is called fillContourLayers in https://github.com/DerZade/meh-utils/blob/master/internal/mvt/buildVectorTiles.go#L239-L278
        fill_contour_layers(lod_layer_names, projection.get_collections_mut()).unwrap_or_else(|e| {
            println!("could not generate contours for lod {}: {}", lod, e);
        });
        build_lod_vector_tiles(projection.get_collections_mut(), world_size, lod, &lod_dir).unwrap_or_else(|err| {
            println!("error when generating vector tiles for lod {}: {}", lod, err);
        });

        println!(
            "✔️  Built mapbox vector tiles LOD {} in {}μs",
            lod,
            now.elapsed().as_micros()
        );

        projection_lod = projection.decrease_lod();
    }

    Ok(())
}

fn create_tile(col: u16, row: u16, collections: &mut HashMap<String, FeatureCollection>) -> anyhow::Result<Tile> {
    //println!("create_tile with col {}, row {}, and {} collections", col, row, collections.len());

    let offset: Coordinate<MvtGeoFloatType> = Coordinate {
        x: (col as f32 * DEFAULT_EXTENT as f32).into(),
        y: (row as f32 * DEFAULT_EXTENT as f32).into(),
    };
    let extent: Coordinate<MvtGeoFloatType> = Coordinate {
        x: DEFAULT_EXTENT.into(),
        y: DEFAULT_EXTENT.into(),
    };

    let tile_border = Rect::new(offset, offset.add(extent));

    let mut layers: Vec<Layer> = vec![];
    collections.iter().for_each(|(name, features)| {
        layers.push(features.to_layer(name.clone(), &tile_border, &offset));
    });

    Ok(Tile::from_layers(layers))
}

fn build_lod_vector_tiles(collections: &mut HashMap<String, FeatureCollection>, world_size: NonZeroUsize, lod: usize, lod_dir: &PathBuf) -> anyhow::Result<()> {
    println!("build_lod_vector_tiles with {} collections, worldsize {} and lod {} into {}", collections.len(), world_size, lod, lod_dir.to_str().unwrap_or("WAT"));

    fn ensure_directory(dir: &PathBuf) -> Result<(), Error>{
        if !dir.is_dir() {
            std::fs::create_dir(dir).map_err(|e| {Error::new(e)})
        } else {
            Ok(())
        }
    }

    let tiles_per_dimension: u16 = (2 as u16).pow(lod as u32);

    ensure_directory(&lod_dir)?;

    for col in 0..tiles_per_dimension {
        let col_path= lod_dir.join(PathBuf::from(col.to_string()));
        ensure_directory(&col_path)?;

        for row in 0..tiles_per_dimension {
            let tile = create_tile(col, row, collections)?;
            let file_path_buf = col_path.join(format!("{}.pbf", row));
            let file_path = file_path_buf.to_str().ok_or(Error::msg("couldnt create filename. wat."))?;
            tile.write_to_file(file_path);
        }
    }

    Ok(())
}

/// there are layers matching `^contours/(\d+)$` . Matching group denotes contour line interval.
/// Example: contours/10 is supposed to contain contour lines in 10m intervals.
/// This function fills the `^contours/\d+$` layers selectively with features from "contours" layer.
fn fill_contour_layers(lod_layer_names: Vec<String>, collections: &mut HashMap<String, FeatureCollection>) -> anyhow::Result<()> {

    let contour_layer = collections.get("contours");

    let contour_features: FeatureCollection = contour_layer
        .ok_or(anyhow::Error::msg("could not find 'contours' layer"))?
        .clone();

    let contours_names: Vec<(String, usize)> = lod_layer_names.iter().filter_map(|name| {
        name.strip_prefix("contours/").and_then(|num_str| {
            let i = num_str.parse::<usize>();
            i.ok()
        }).map(|i| {
            (name.to_string(), i)
        })
    }).filter(|(_, interval)| {
        interval != &0
    }).collect();

    contours_names.iter().for_each(|(name, elevation)| {
        let features = contour_features.clone();
        features.iter().step_by(*elevation).for_each(|f| {
            collections.get_mut(name).unwrap().push(f.clone());
        });
    });
    Ok(())
}

fn simplify_mounts(_: &mut FeatureCollection, _: f64) {
    todo!("mount simplification");
}