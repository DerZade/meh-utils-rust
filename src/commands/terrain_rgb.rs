use anyhow::bail;
use clap::{arg, App};
use image::{DynamicImage, Rgb, RgbImage};

use crate::commands::Command;
use crate::dem::{DEMParser, DEMRaster};
use crate::utils::{build_tile_set, calc_max_lod};

use std::fs::File;
use std::path::Path;

use std::io::{BufReader, Read};
use std::time::Instant;

use flate2::bufread::GzDecoder;

pub struct TerrainRGB {}

impl Command for TerrainRGB {
    fn register(&self) -> App<'static> {
        App::new("terrain_rgb")
            .about("Build Terrain-RGB tiles from grad_meh data.")
            .arg(arg!(-i --input <INPUT_DIR> "Path to grad_meh map directory"))
            .arg(arg!(-o --output <OUTPUT_DIR> "Path to output directory"))
    }
    fn run(&self, args: &clap::ArgMatches) -> anyhow::Result<()> {
        let start = Instant::now();

        let input_path_str = args.value_of("input").unwrap();
        let output_path_str = args.value_of("output").unwrap();

        let input_path = Path::new(input_path_str);
        let output_path = Path::new(output_path_str);

        if !output_path.is_dir() {
            bail!("Output path is not a directory");
        }

        println!("▶️  Loading meta.json");
        let meta_path = input_path.join("meta.json");
        let meta = crate::metajson::from_file(&meta_path)?;
        println!("✔️  Loaded meta.json");

        let now = Instant::now();
        println!("▶️  Loading DEM");
        let dem_path = input_path.join("dem.asc.gz");
        if !dem_path.is_file() {
            bail!("Couldn't find dem.asc.gz");
        }
        let dem = load_dem(&dem_path)?;
        println!("✔️  Loaded DEM in {}ms", now.elapsed().as_millis());

        let elevation_offset = meta.elevation_offset;

        let img = calculate_image(elevation_offset, &dem)?;

        let max_lod = calc_max_lod(&img);
        println!("ℹ️  Calculated max lod: {}", max_lod);

        let now = Instant::now();
        println!("▶️  Building tiles");
        for lod in 0..max_lod + 1 {
            let now = Instant::now();
            build_tile_set(&output_path, &img, lod)?;
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

        println!("\n    🎉  Finished in {}ms", start.elapsed().as_millis());

        Ok(())
    }
}

fn load_dem(path: &Path) -> anyhow::Result<DEMRaster> {
    let file = File::open(path)?;

    let buf = BufReader::new(file);
    let mut dec = GzDecoder::new(buf);
    let mut s = String::new();

    dec.read_to_string(&mut s)?;

    let slice = &s[..];

    let raster = DEMParser::parse(slice)?;

    Ok(raster)
}

fn calculate_image(elevation_offset: f32, dem: &DEMRaster) -> anyhow::Result<DynamicImage> {
    let (w, h) = dem.dimensions();
    let mut buffer = RgbImage::new(w as u32, h as u32);

    for x in 0..w {
        for y in 0..h {
            let elev = dem.z(x, y) + elevation_offset;
            let pixel = elevation_to_rgb(elev);
            buffer.put_pixel(x as u32, y as u32, pixel);
        }
    }

    Ok(DynamicImage::ImageRgb8(buffer))
}

/*
    The Mapbox Terrain-RGB Tiles use the following equation to decode
    height values from rgb.

    height = -10000 + ((R * 256 * 256 + G * 256 + B) * 0.1)

    To make things easier we'll replace (R * 256 * 256 + G * 256 + B) with x to get the following equation:
    height = -10000 + (x * 0.1)
    now we can solve the equation for x and get:
    x = 10 * height + 100000

    To get the r, g and b value from x we'll use a little trick:
    We could write (R * 256 * 256 + G * 256 + B) as (R * 256^2 + G * 256^1 + B * 256^0)
    That should ring a bell for every computer scientist. Looks a awful lot like a numeral system conversion from Base256
    So we'll just convert x to as Base256 number. Position 2 will be r, position 1 will be g and position 0 will be b
*/
const MAX_X: i64 = 256_i64.pow(3) - 1;

fn elevation_to_rgb(elevation: f32) -> Rgb<u8> {
    let mut x = (10.0 * elevation) as i64 + 100000 % MAX_X;

    let b = (x % 256) as u8;
    x = x / 256;

    let g = (x % 256) as u8;
    x = x / 256;

    let r = (x % 256) as u8;

    Rgb([r, g, b])
}
