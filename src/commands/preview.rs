use clap::{arg, App};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::commands::{ Command };

use std::{path::Path};
use image::{io::Reader as ImageReader, DynamicImage, codecs::png::PngEncoder};

use std::io::{Error, ErrorKind, BufWriter};
use std::fs::File;
use image::GenericImageView;
use std::time::{Instant};

pub struct Preview {}

impl Command for Preview {
    fn register(&self) -> App<'static> {
        App::new("preview")
            .about("Build resolutions for preview image.")
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
            return Err(Box::new(Error::new(ErrorKind::Other, "Output path is not a directory")))
        }

        let preview_path = input_path.join("preview.png");
        if !preview_path.is_file() {
            return Err(Box::new(Error::new(ErrorKind::NotFound, "Couldn't find preview.png")))
        }

        let now = Instant::now();
    	println!("▶️  Loading preview image");
        let img = ImageReader::open(preview_path)?.decode()?;
        println!("✔️  Loaded preview image in {}ms", now.elapsed().as_millis());

        let now = Instant::now();
    	println!("▶️  Writing original preview image to output");
        if let Err(e) = encode_png(&output_path.join("preview.png"), &img) {
            println!("❌  Failed to write original preview image");
            println!("{}", e);
        } else {
            println!("✔️  Wrote original preview image in {}ms", now.elapsed().as_millis());
        }

        [128u32, 256, 512, 1024].par_iter().for_each(|size| {
            let now = Instant::now();
            println!("▶️  Building x{} image", size);

            let thumb = img.thumbnail(*size, *size);
            let thumb_path = output_path.join(format!("preview_{}.png",size));

            if let Err(e) = encode_png(&thumb_path, &thumb) {
		        println!("❌  Build of x{} failed", size);
                println!("{}", e);
            } else {
                println!("✔️  Built x{} in {}ms", size, now.elapsed().as_millis())
            }
        });

        println!("\n    🎉  Finished in {}ms", start.elapsed().as_millis());


        Ok(())
    }
}

fn encode_png(file_path: &Path, img: &DynamicImage) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(file_path).unwrap();
    let ref mut buf = BufWriter::new(file);
    let encoder = PngEncoder::new(buf);

    let dim = img.dimensions();
    match encoder.encode(&img.to_bytes(), dim.0, dim.1, img.color()) {
      Ok(_) => Ok(()),
      Err(err) => Err(Box::new(Error::new(ErrorKind::Other, err.to_string())))
    }
}