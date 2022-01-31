use std::collections::HashMap;
use crate::dem::DEMRaster;
use crate::feature::{FeatureCollection, Feature, PropertyValue};

const NEIGHBOUR_CELLS: [(i32, i32); 8] = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

pub fn build_mounts(dem: &DEMRaster, elevation_offset: f32, collections: &mut HashMap<String, FeatureCollection<f32>>) -> anyhow::Result<()> {

    let (w, h) = dem.dimensions();
    let mut mounts = FeatureCollection::<f32>::new();

    for col in 1..w-1 {
        for row in 1..h-1 {
            let elev = dem.z(col, row);

			// we'll only create mounts for peaks, which are above the water level
            if elev <= 0.0 {
                continue;
            }

            let has_higher_neighbors = NEIGHBOUR_CELLS.iter().any(|(x, y)| {
                let comp_elev = dem.z((col as i32 + x) as usize, (row as i32 + y) as usize);

                // we'll count same elevation as a high neighbour because we don't
                // want to generate a "mount" for cells that are in the middle of a plane
                return comp_elev >= elev;
            });

            // add mount if all neighbours are lower (= this is a peak)
            if !has_higher_neighbors {
                let geometry = geo::Point::new(dem.x(col), dem.y(col));

                let corrected_elevation = elev + elevation_offset;
                let corrected_elevation_str = format!("{:.0}", corrected_elevation).to_string();
                mounts.push(
                    Feature {
                        geometry: geometry.into(),
                        properties: HashMap::from([
                            ("elevation".to_string(), PropertyValue::Number(corrected_elevation)),
                            ("text".to_string(), PropertyValue::String(corrected_elevation_str))
                        ])
                    }
                );
            }
        }
    }

    // sort mounts to make sure simplifying works as intended,
    // the higher the elevations, the more important it is 
    mounts.sort_by(|a, b| {
        let elev_a = a.properties.get("elevation").unwrap();
        let elev_b = b.properties.get("elevation").unwrap();
        elev_a.partial_cmp(elev_b).unwrap()
    });

    collections.insert("mounts".to_string(), mounts);

    Ok(())
}