use std::{fs::create_dir_all, panic, path::Path};

use image::{imageops, DynamicImage, GenericImageView, Rgba};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::{encode_png, TileError, TILE_SIZE_IN_PX};

pub fn build_tile_set(set_base_path: &Path, img: &DynamicImage, lod: usize) -> anyhow::Result<()> {
    let tiles_per_row_col = 2u32.pow(lod as u32);

    // generate all column directories
    (0..tiles_per_row_col)
        .into_par_iter()
        .panic_fuse()
        .for_each(|col| {
            let file_path = set_base_path.join(lod.to_string()).join(col.to_string());
            create_dir_all(file_path).unwrap();
        });

    let (width, height) = img.dimensions();

    let tile_width = width / tiles_per_row_col;
    let tile_height = height / tiles_per_row_col;

    let width_remainder = width % tiles_per_row_col;
    let height_remainder = height % tiles_per_row_col;

    let result = panic::catch_unwind(|| {
        (0..tiles_per_row_col * tiles_per_row_col)
            .into_par_iter()
            .panic_fuse()
            .for_each(|index| {
                let col = index / tiles_per_row_col;
                let row = index % tiles_per_row_col;
                let x = tile_width * col;
                let y = tile_height * row;
                let mut w = tile_width;
                let mut h = tile_height;

                // distribute remaining pixels over the first X rows / cols
                if width_remainder > col + 1 {
                    w += 1;
                }
                if height_remainder > row + 1 {
                    h += 1;
                }

                let sub = img.view(x, y, w, h);
                let resized = resize(&sub);

                match write_tile(set_base_path, &resized, col, row, lod) {
                    Ok(_) => {}
                    Err(e) => panic::panic_any(TileError::new(col, row, e)),
                };
            });
    });

    result.map_err::<anyhow::Error, _>(|e| {
        let tile_error = e.downcast_ref::<TileError>().unwrap();
        anyhow::anyhow!("{}", tile_error)
    })
}

fn resize<I: GenericImageView<Pixel = Rgba<u8>>>(image: &I) -> DynamicImage {
    let buffer = imageops::resize(
        image,
        TILE_SIZE_IN_PX,
        TILE_SIZE_IN_PX,
        image::imageops::FilterType::Triangle,
    );

    DynamicImage::ImageRgba8(buffer)
}

fn write_tile(
    set_base_path: &Path,
    img: &DynamicImage,
    x: u32,
    y: u32,
    z: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file_path = set_base_path
        .join(z.to_string())
        .join(x.to_string())
        .join(format!("{}.png", y));
    encode_png(&file_path, img)
}
