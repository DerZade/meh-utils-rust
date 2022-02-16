use std::{collections::HashMap, path::Path, fs::{DirEntry, read_dir, File}, io::{BufReader}};
use std::convert::TryInto;

use anyhow::bail;
use flate2::bufread::GzDecoder;
use geo::{Geometry};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::feature::{FeatureCollection, Feature, PropertyValue};

// one possible implementation of walking a directory only visiting files
fn find_files_rec(dir: &Path) -> anyhow::Result<Vec<DirEntry>> {
    let mut files = Vec::<DirEntry>::new();

    if dir.is_dir() {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                files.extend(find_files_rec(&path)?);
            } else {
                if entry.file_name().to_str().unwrap().ends_with(".geojson.gz") {
                    files.push(entry);
                }
            }
        }
    }
    Ok(files)
}

pub fn load_geo_jsons(input_path: &Path, collections: &mut HashMap<String, FeatureCollection>) -> anyhow::Result<()> {

    let (mut ok_results, mut err_results): (Vec<_>, Vec<_>) = find_files_rec(input_path)?.into_par_iter().map(|entry| -> anyhow::Result<(String, FeatureCollection)> {
        let path_buf = entry.path();
        let path = path_buf.as_path();
        let layer_name = path_to_layer_name(path, input_path)?;
        let fc = read_zipped_geo_json(path)?;

        Ok((layer_name, fc))
    }).partition(Result::is_ok);

    if err_results.len() > 0 {
        return Err(err_results.remove(0).err().unwrap());
    }

    let mut values: Vec<_> = ok_results.drain(0..ok_results.len()).map(|r| r.unwrap()).collect();

    values.drain(0..values.len()).for_each(|(name, fc)| {
        collections.insert(name, fc);
    });

    Ok(())
}

fn path_to_layer_name (file_path: &Path, input_path: &Path) -> anyhow::Result<String> {
    let rel_path = file_path.strip_prefix(input_path)?;

    let s = match rel_path.to_str() {
        Some(val) => val,
        None => bail!("Could not generate layer name"),
    };

    let string = s.to_string().replace(".geojson.gz", "");

    Ok(string)
}

fn read_zipped_geo_json(path: &Path) -> anyhow::Result<FeatureCollection> {
    let file = File::open(path)?;

    let buf = BufReader::new(file);
    let dec = GzDecoder::new(buf);

    let mut geojson_features: Vec<geojson::Feature> = serde_json::from_reader(dec)?;

    let fc: FeatureCollection = geojson_features.drain(0..geojson_features.len()).filter_map(|f|{
        if f.geometry.is_none() {
            return None
        }

        let gj_geo = f.geometry.unwrap();
        let geometry: Geometry<f32> = gj_geo.try_into().unwrap();


        let properties: HashMap<_, _> = match f.properties {
            Some(map) => {
                map.into_iter().map(|(key, val)| -> (String, PropertyValue) {
                    (key.clone(), val.into())
                }).collect()
            },
            None => HashMap::new(),
        };

        Some(Feature { geometry, properties })
    }).collect();

    println!("geojson_features has {} elements.", geojson_features.len());

    Ok(fc)
}