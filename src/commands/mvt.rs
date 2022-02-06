use std::cmp::Ordering;
use anyhow::{bail, Error};
use num_traits::cast::ToPrimitive;

use geo::map_coords::MapCoordsInplace;
use geo::{Coordinate, CoordNum, GeoFloat, Geometry, LineString};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::dem::{DEMRaster, load_dem};
use crate::feature::{FeatureCollection, Simplifiable};
use crate::mvt::{load_geo_jsons, build_mounts, find_lod_layers};

use std::collections::HashMap;
use std::iter::Sum;
use std::path::Path;

use std::time::Instant;
use contour::ContourBuilder;
use geo::Geometry::Point;
use geojson::{Feature, PolygonType, Value};
use mapbox_vector_tile::Layer;
use crate::feature::Feature as CrateFeature;
use crate::metajson::{MetaJsonParser};

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;
    use geo::{Coordinate};
    use geojson::{Geometry, Value};
    use geojson::Feature;
    use geojson::Value::{MultiPolygon};
    use rand::{Rng, thread_rng};
    use crate::commands::mvt::{build_contours, build_vector_tiles, fill_contour_layers, MapboxVectorTiles, try_from_geojson_feature_for_crate_feature, try_from_geojson_value_for_geo_geometry, vec_f64_to_coordinate_f32};
    use crate::dem::{DEMRaster, Origin};
    use crate::feature::{Feature as CrateFeature, FeatureCollection};
    use crate::metajson::DummyMetaJsonParser;
    use crate::test::with_input_and_output_paths;

    #[test]
    fn bails_on_input_dir_empty() {
        with_input_and_output_paths(|input_path, output_path| {
            let result = (MapboxVectorTiles::new(Box::new(DummyMetaJsonParser { succeeds: true }))).exec(&input_path, &output_path);
            assert!(result.is_err());
        });
    }

    #[ignore]
    #[test]
    fn runs_successfully() {
        with_input_and_output_paths(|_, output_path| {
            let input_path = Path::new("./resources/test/happy/input").to_path_buf();
            let result =  (MapboxVectorTiles::new(Box::new(DummyMetaJsonParser { succeeds: true }))).exec(&input_path, &output_path);
            assert!(result.is_ok());
        });
    }


    #[test]
    fn build_contours_does_its_thing() {
        let contour_line_to_vec_of_tuple = |feature: &CrateFeature<f32>| -> Vec<(f32, f32)> {
            match &feature.geometry {
                geo::Geometry::MultiPolygon(foo) => {
                    let poly = foo.0.get(0).unwrap();
                    let ext = poly.exterior();
                    ext.0.iter().map(|f| { (f.x.clone(), f.y.clone()) }).collect()
                },
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
        let mut collections: HashMap<String, crate::feature::FeatureCollection<f32>> = HashMap::new();

        let res = build_contours(&raster, 0.0, 2048, 2, &mut collections);

        assert!(res.is_ok());
        assert_eq!(collections.len(), 1);
        assert!(collections.contains_key("contour_lines"));
        let contour_lines: &FeatureCollection<f32> = collections.get("contour_lines").unwrap();
        assert_eq!(contour_lines.len(), 5);
        println!("ookay collection: {}", collections.get("contour_lines").unwrap().0.len());

        let v = contour_line_to_vec_of_tuple(contour_lines.0.get(0).unwrap());

        assert_eq!(v, vec![
            (5.0, 5.5), (5.0, 4.5), (5.0, 3.5), (5.0, 2.5), (5.0, 1.5),
            (5.0, 0.5), (4.5, 0.0), (3.5, 0.0), (2.5, 0.0), (1.5, 0.0),
            (0.5, 0.0), (0.0, 0.5), (0.0, 1.5), (0.0, 2.5), (0.0, 3.5),
            (0.0, 4.5), (0.0, 5.5), (0.5, 6.0), (1.5, 6.0), (2.5, 6.0),
            (3.5, 6.0), (4.5, 6.0), (5.0, 5.5)
        ]);

        let v = contour_line_to_vec_of_tuple(contour_lines.0.get(1).unwrap());

        assert_eq!(v, vec![
            (4.0, 4.5), (4.0, 3.5), (4.0, 2.5), (3.5, 2.0), (3.0, 1.5),
            (2.5, 1.0), (1.5, 1.0), (1.0, 1.5), (1.0, 2.5), (1.0, 3.5),
            (1.0, 4.5), (1.5, 5.0), (2.5, 5.0), (3.5, 5.0), (4.0, 4.5)
        ]);

        let v = contour_line_to_vec_of_tuple(contour_lines.0.get(2).unwrap());

        assert_eq!(v, vec![
            (3.0, 4.5), (3.5, 4.0), (4.0, 3.5), (3.5, 3.0), (3.0, 2.5),
            (2.5, 2.0), (1.5, 2.0), (1.0, 2.5), (1.0, 3.5), (1.0, 4.5),
            (1.5, 5.0), (2.5, 5.0), (3.0, 4.5)
        ]);

        let v = contour_line_to_vec_of_tuple(contour_lines.0.get(3).unwrap());

        assert_eq!(v, vec![
            (2.0, 4.5), (2.0, 3.5), (2.0, 2.5), (1.5, 2.0), (1.0, 2.5),
            (1.0, 3.5), (1.0, 4.5), (1.5, 5.0), (2.0, 4.5)
        ]);

        let v = contour_line_to_vec_of_tuple(contour_lines.0.get(4).unwrap());

        assert_eq!(v, vec![
            (2.0, 3.5), (1.5, 3.0), (1.0, 3.5), (1.5, 4.0), (2.0, 3.5)
        ]);
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

        let cratefeature: anyhow::Result<CrateFeature<f32>> = try_from_geojson_feature_for_crate_feature(geojsonfeature);

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
    #[ignore]
    fn build_vector_tiles_does_not_explode_on_0_lods() {
        with_input_and_output_paths(|_, output_path| {
            let res = build_vector_tiles(&output_path, HashMap::<String, FeatureCollection<f32>>::new(), 0, 1);

            assert!(res.is_ok());
        });
    }

    #[test]
    #[ignore]
    fn build_vector_tiles_does_not_explode_on_empty_input() {
        with_input_and_output_paths(|_, output_path| {
            let res = build_vector_tiles(&output_path, HashMap::<String, FeatureCollection<f32>>::new(), 1, 1);

            assert!(res.is_ok());
        });
    }

    fn some_feature() -> CrateFeature<f32> {
        let mut rng = thread_rng();
        let mut rand = || {rng.gen_range(0.0..127.0)};
        CrateFeature {
            geometry: geo::Geometry::Point(geo::Point(Coordinate {x:  rand(), y: rand()})),
            properties: HashMap::new(),
        }
    }

    fn collections_with_layers(layer_names: Vec<&str>/*, add_features: bool*/) -> HashMap<String, FeatureCollection<f32>> {
        let mut collections = HashMap::new();
        layer_names.iter().for_each(|layer_name| {
            let collection = FeatureCollection::from_iter(vec![]);
            collections.insert(layer_name.to_string(), collection);
        });

        collections
    }

    #[test]
    fn fill_contour_layers_does_not_panic_if_no_contours_but_returns_err() {
        let mut layers = HashMap::<String, FeatureCollection<f32>>::new();
        let res = fill_contour_layers(vec!["foo".to_string(), "contours/1".to_string()], &mut layers);

        assert!(res.is_err());
    }

    #[test]
    fn fill_contour_layers_copies_all_features_from_contours_to_contours_1() {
        let mut layers = collections_with_layers(vec!["contours", "contours/1", "foo"]);
        layers.get_mut("foo").unwrap().push(some_feature());
        layers.get_mut("contours").unwrap().push(some_feature());
        layers.get_mut("contours").unwrap().push(some_feature());

        fill_contour_layers(layers.keys().map(|f| {f.to_string()}).collect(), &mut layers);

        let contours_1_features = &layers.get("contours/1").unwrap().0;
        let contours_features = &layers.get("contours").unwrap().0;
        assert_eq!(2, contours_1_features.len());
        for i in 0..=1 {
            assert_eq!(contours_features.get(i).unwrap().geometry, contours_1_features.get(i).unwrap().geometry);
        }
        assert_eq!(1, layers.get("foo").unwrap().len());
    }

    #[test]
    fn fill_contour_layers_copies_only_every_fifth_feature_from_contours_to_contours_5() {
        let mut layers = collections_with_layers(vec!["contours", "contours/5", "foo"]);
        for _ in 0..11 {
            layers.get_mut("contours").unwrap().push(some_feature());
        }

        fill_contour_layers(layers.keys().map(|f| {f.to_string()}).collect(), &mut layers);

        let contours_5_features = &layers.get("contours/5").unwrap().0;
        let contours_features = &layers.get("contours").unwrap().0;
        assert_eq!(3, contours_5_features.len());
        assert_eq!(contours_features.get(0).unwrap().geometry, contours_5_features.get(0).unwrap().geometry);
        assert_eq!(contours_features.get(5).unwrap().geometry, contours_5_features.get(1).unwrap().geometry);
        assert_eq!(contours_features.get(10).unwrap().geometry, contours_5_features.get(2).unwrap().geometry);
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
        build_contours(&dem, meta.elevation_offset, meta.world_size, 10, &mut collections)?;
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



fn build_contours(dem: &DEMRaster, elevation_offset: f32, _: u32, step: usize, collections: &mut HashMap<String, FeatureCollection<f32>>) -> anyhow::Result<()> {
    let cmp = |a: &&f64, b: &&f64| -> Ordering {a.partial_cmp(b).unwrap()};

    let elevation_offset_f64 = elevation_offset.to_f64().unwrap();
    let dem64 = dem
        .get_data()
        .iter()
        .map(|x| { (elevation_offset + x).to_f64().unwrap()})
        .collect::<Vec<f64>>();

    let no_data_value: f64 = dem.get_no_data_value().to_f64().unwrap();
    let min_elevation = dem64.iter().filter(|x| {*x != &no_data_value}).min_by(cmp).map(|f| {f.to_i64().unwrap()}).ok_or(anyhow::Error::msg("no values in DEM raster"))?;
    let max_elevation = dem64.iter().filter(|x| {*x != &no_data_value}).max_by(|a, b| -> Ordering {a.partial_cmp(b).unwrap()}).map(|f| {f.to_i64().unwrap()}).ok_or(anyhow::Error::msg("no values in DEM raster"))?;
    // hmm how do we use worldsize? do we?

    let builder = ContourBuilder::new(dem.dimensions().0 as u32, dem.dimensions().1 as u32, false);
    let thresholds: Vec<f64> = (min_elevation..=max_elevation).step_by(step).map(|f| {f.to_f64().unwrap() + elevation_offset_f64}).collect();

    println!("## contour builder ##");
    println!("dimensions: {} by {}", dem.dimensions().0 as u32, dem.dimensions().1 as u32);
    println!("all thresholds: {}", thresholds.iter().map(|f| {f.to_string()}).collect::<Vec<String>>().join(" "));
    println!("elevation offset: {}", elevation_offset_f64);
    println!("elevation offset: {}", elevation_offset_f64);

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

fn build_vector_tiles<T: CoordNum + Send + GeoFloat + From<f32> + Sum>(output_path: &Path, mut collections: HashMap<String, FeatureCollection<T>>, max_lod: u8, world_size: u32) -> anyhow::Result<()> {

    let world_size_f32 = world_size as f32;
    let tiles_per_col_row = 2_u32.pow(max_lod as u32);
    let pixels = tiles_per_col_row as u64 * TILE_SIZE;
    let factor = pixels as f32 / world_size_f32;

    let factor_t: T = factor.into();
    let world_size_t: T = world_size_f32.into();

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
        });

        let lod_layer_names = find_lod_layers(&collections, lod);
        // note: the following is called fillContourLayers in https://github.com/DerZade/meh-utils/blob/master/internal/mvt/buildVectorTiles.go#L239-L278
        fill_contour_layers(lod_layer_names, &mut collections).unwrap_or_else(|e| {
            println!("could not generate contours for lod {}: {}", lod, e);
        });
        todo!("build LOD vector tiles");
        build_lod_vector_tiles(&mut collections, world_size, lod);
    }

    Ok(())
}

fn build_lod_vector_tiles<T: CoordNum>(collections: &mut HashMap<String, FeatureCollection<T>>, world_size: u32, lod: u8) -> anyhow::Result<()> {
    Ok(())
}

/// there are layers matching `^contours/(\d+)$` . Matching group denotes contour line interval.
/// Example: contours/10 is supposed to contain contour lines in 10m intervals.
/// This function fills the `^contours/\d+$` layers selectively with features from "contours" layer.
fn fill_contour_layers<T: CoordNum>(lod_layer_names: Vec<String>, collections: &mut HashMap<String, FeatureCollection<T>>) -> anyhow::Result<()> {
    let contour_features = collections.get_mut("contours").ok_or(anyhow::Error::msg("foo"))?.clone();
    let contours_names: Vec<(String, usize)> = lod_layer_names.iter().map(|name| {
        let x = name.strip_prefix("contours/");
        let n = x.map_or(0, |s| {
            let i = s.parse::<usize>();
            i.map_or(0, |n| {n})
        });
        (name.to_string(), n)
    }).filter(|(_, interval)| {
        interval != &0
    }).collect();

    contours_names.iter().for_each(|contour| {
        let features = contour_features.clone();
        features.iter().step_by(contour.1).for_each(|f| {
            collections.get_mut(&contour.0).unwrap().push(f.clone());
        });
    });
    Ok(())
}

fn project_layers_in_place<T: CoordNum, F: Fn(&(T, T)) -> (T, T) + Copy>(layers: &mut HashMap<String, FeatureCollection<T>>, transform: F) {
    for (_, layer) in layers.iter_mut() {
        layer.map_coords_inplace(transform);
    }
}

fn simplify_mounts<T: CoordNum>(_: &mut FeatureCollection<T>, _: f64) {
    todo!("mount simplification");
}