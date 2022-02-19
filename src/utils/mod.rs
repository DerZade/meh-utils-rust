mod build_tile_set;
mod tile_error;

use image::{codecs::png::PngEncoder, DynamicImage, GenericImageView};
use std::fs::File;
use std::io::{BufWriter, Error, ErrorKind};
use std::path::Path;

pub use build_tile_set::build_tile_set;
pub use tile_error::TileError;

pub const TILE_SIZE_IN_PX: u32 = 256;

pub fn calc_max_lod(image: &DynamicImage) -> usize {
    let width = image.dimensions().0 as f32;

    let tiles_per_row = (width / TILE_SIZE_IN_PX as f32).ceil();

    return tiles_per_row.log2().ceil() as usize;
}

pub fn encode_png(
    file_path: &Path,
    img: &DynamicImage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = File::create(file_path)?;
    let ref mut buf = BufWriter::new(file);
    let encoder = PngEncoder::new(buf);

    let dim = img.dimensions();
    match encoder.encode(&img.to_bytes(), dim.0, dim.1, img.color()) {
        Ok(_) => Ok(()),
        Err(err) => Err(Box::new(Error::new(ErrorKind::Other, err.to_string()))),
    }
}
