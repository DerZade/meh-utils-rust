mod parser;
mod raster;

use flate2::bufread::GzDecoder;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

pub use parser::DEMParser;
pub use raster::DEMRaster;
pub use raster::Origin;

pub fn load_dem(path: &Path) -> anyhow::Result<DEMRaster> {
    let file = File::open(path)?;

    let buf = BufReader::new(file);
    let mut dec = GzDecoder::new(buf);
    let mut s = String::new();

    dec.read_to_string(&mut s)?;

    let slice = &s[..];

    let raster = DEMParser::parse(slice)?;

    Ok(raster)
}
