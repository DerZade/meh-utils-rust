use anyhow::bail;
use geo::map_coords::MapCoordsInplace;
use geo::{CoordNum, GeoFloat};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::commands::{MehDataCommand};
use crate::dem::{DEMRaster, load_dem};
use crate::feature::{FeatureCollection, Simplifiable};
use crate::mvt::{load_geo_jsons, build_mounts};

use std::collections::HashMap;
use std::path::Path;

use std::time::Instant;
use crate::metajson::{MetaJsonParser};

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use crate::commands::{MapboxVectorTiles, MehDataCommand};
    use crate::metajson::DummyMetaJsonParser;
    use crate::utils::with_input_and_output_paths;

    #[test]
    fn bails_on_input_dir_empty() {
        with_input_and_output_paths(|input_path, output_path| {
            let result = (MapboxVectorTiles::new(Box::new(DummyMetaJsonParser { succeeds: true }))).exec(&input_path, &output_path);
            assert!(result.is_err());
        });
    }
}

pub struct MapboxVectorTiles {
    meta_json: Box<dyn MetaJsonParser>,
}
impl MapboxVectorTiles {
    pub fn new(meta_json: Box<dyn MetaJsonParser>) -> Self {
        MapboxVectorTiles { meta_json }
    }
}
impl MehDataCommand for MapboxVectorTiles {
    fn get_description(&self) -> &str {
        "Build mapbox vector tiles from grad_meh data."
    }

    fn exec(&self, input_path: &Path, output_path: &Path) -> anyhow::Result<()> {
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
        let dem = load_dem(&dem_path)?;
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

fn build_contours<T: CoordNum>(_dem: &DEMRaster, _elevation_offset: f32, _world_size: u32, _collections: &mut HashMap<String, FeatureCollection<T>>) -> anyhow::Result<()> {
    // TODO
    Ok(())
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
        let lod_dir = output_path.join(lod.to_string());
        let start = Instant::now();

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

fn simplify_mounts<T: CoordNum>(collection: &mut FeatureCollection<T>, threshold: f64) {
    todo!();
}