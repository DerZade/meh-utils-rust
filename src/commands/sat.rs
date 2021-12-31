use clap::{arg, App};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use std::io::{Error, ErrorKind};
use std::path::Path;
use std::time::Instant;

use image::{imageops::replace, io::Reader as ImageReader, DynamicImage, GenericImageView};

use crate::commands::Command;
use crate::utils::{build_tile_set, calc_max_lod, TileError};

pub struct Sat {}

impl Command for Sat {
    fn register(&self) -> App<'static> {
        App::new("sat")
            .about("Build satellite tiles from grad_meh data.")
            .arg(arg!(-i --input <INPUT_DIR> "Path to grad_meh map directory"))
            .arg(arg!(-o --output <OUTPUT_DIR> "Path to output directory"))
    }
    fn run(&self, args: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();

        let input_path_str = args.value_of("input").unwrap();
        let output_path_str = args.value_of("output").unwrap();

        let input_path = Path::new(input_path_str);
        let output_path = Path::new(output_path_str);

        if !output_path.is_dir() {
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "Output path is not a directory",
            )));
        }

        println!("▶️  Loading meta.json");
        let meta_path = input_path.join("meta.json");
        let meta = crate::metajson::from_file(&meta_path)?;
        println!("✔️  Loaded meta.json");

        let now = Instant::now();
        println!("▶️  Combining satellite image");
        let combined_sat_image = load_combined_sat_image(input_path)?;
        println!(
            "✔️  Combined satellite image in {}ms",
            now.elapsed().as_millis()
        );

        let max_lod = calc_max_lod(&combined_sat_image);
        println!("ℹ️  Calculated max lod: {}", max_lod);

        let now = Instant::now();
        println!("▶️  Building tiles");
        for lod in 0..max_lod + 1 {
            let now = Instant::now();
            build_tile_set(&output_path, &combined_sat_image, lod)?;
            println!(
                "    ✔️  Finished tiles for LOD {} in {}ms",
                lod,
                now.elapsed().as_millis()
            );
        }
        println!(
            "✔️  Built satellite tiles in {}ms",
            now.elapsed().as_millis()
        );

        let now = Instant::now();
        println!("▶️  Creating tile.json");
        crate::tilejson::write(output_path, max_lod, meta, "Satellite", Vec::new())?;
        println!("✔️  Created tile.json in {}ms", now.elapsed().as_millis());

        println!("\n    🎉  Finished in {}ms", start.elapsed().as_millis());

        Ok(())
    }
}

fn load_combined_sat_image(input_path: &Path) -> Result<DynamicImage, Box<dyn std::error::Error>> {
    let sat_path = input_path.join("sat");

    let now = Instant::now();

    let results: Vec<_> = (0..16)
        .into_par_iter()
        .map(|index| {
            let col = index / 4;
            let row = index % 4;

            let img_path = sat_path.join(col.to_string()).join(format!("{}.png", row));

            ImageReader::open(img_path)
                .map_err(|e| TileError::new(col, row, e))?
                .decode()
                .map_err(|e| TileError::new(col, row, e))
        })
        .collect();

    let (ok_results, err_results): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);

    if err_results.len() > 0 {
        let error_string: Vec<_> = err_results
            .into_iter()
            .map(|r| format!("\t{}", r.err().unwrap()))
            .collect();

        return Err(Box::new(Error::new(
            ErrorKind::Other,
            format!(
                "Failed to load (multiple) tile(s):\n{}",
                error_string.join("\n")
            ),
        )));
    }

    let images: Vec<DynamicImage> = ok_results.into_iter().map(|r| r.unwrap()).collect();
    println!("    ✔️  Loaded tiles in {}ms", now.elapsed().as_millis());

    let mut widths = [0u32; 4];
    let mut heights = [0u32; 4];
    for col in 0..4 {
        for row in 0..4 {
            let (w, h) = images[col * 4 + row].dimensions();

            if widths[col] < w {
                widths[col] = w
            }
            if heights[row] < h {
                heights[row] = h
            }
        }
    }

    let combined_width: u32 = widths.iter().sum();
    let combined_height: u32 = heights.iter().sum();

    let mut combined_image = DynamicImage::new_rgba8(combined_width, combined_height);

    let now = Instant::now();
    for col in 0..4 {
        for row in 0..4 {
            let img = &images[col * 4 + row];
            let x = widths.iter().take(col).sum();
            let y = heights.iter().take(row).sum();

            replace(&mut combined_image, img, x, y);
        }
    }
    println!("    ✔️  Combined tiles in {}ms", now.elapsed().as_millis());

    Ok(combined_image)
}
