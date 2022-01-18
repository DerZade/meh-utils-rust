use anyhow::bail;
use clap::{arg, App};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::commands::Command;
use crate::utils::encode_png;

use image::io::Reader as ImageReader;
use std::path::Path;

use std::time::Instant;

pub struct Preview {}

mod tests {
    use std::path::Path;
    use crate::Command;
    use crate::commands::Preview;

    #[test]
    fn register_returns_app() {
        assert_eq!("preview", (Preview {}).register().get_name());
    }

    #[test]
    fn exec_bails_if_input_or_output_dirs_do_not_exist() {
        assert!((Preview {}).exec(&Path::new("./resources/test/happy/input"), &Path::new("yolo")).is_err());
        assert!((Preview {}).exec(&Path::new("yolo"), &Path::new("./resources/test/happy/output")).is_err());
    }

    #[test]
    fn exec_bails_if_input_preview_file_does_not_exist() {
        assert!((Preview {}).exec(&Path::new("./resources/test/preview_missing/input"), &Path::new("./resources/test/happy/output")).is_err());
    }

    #[test]
    fn exec_bails_if_input_preview_img_is_invalid() {
        assert!((Preview {}).exec(&Path::new("./resources/test/preview_invalid/input"), &Path::new("./resources/test/happy/output")).is_err());
    }

    #[test]
    fn exec_runs_if_prerequisites_are_met() {
        assert!((Preview {}).exec(&Path::new("./resources/test/happy/input"), &Path::new("./resources/test/happy/output")).is_ok());
    }
}

impl Command for Preview {
    fn register(&self) -> App<'static> {
        App::new("preview")
            .about("Build resolutions for preview image.")
            .arg(arg!(-i --input <INPUT_DIR> "Path to grad_meh map directory"))
            .arg(arg!(-o --output <OUTPUT_DIR> "Path to output directory"))
    }
    fn run(&self, args: &clap::ArgMatches) -> anyhow::Result<()> {

        let input_path_str = args.value_of("input").unwrap();
        let output_path_str = args.value_of("output").unwrap();

        let input_path = Path::new(input_path_str);
        let output_path = Path::new(output_path_str);

        self.exec(input_path, output_path)
    }
}
impl Preview {
    fn exec(&self, input_path: &Path, output_path: &Path) -> anyhow::Result<()> {
        let start = Instant::now();

        if !output_path.is_dir() {
            bail!("Output path is not a directory");
        }

        let preview_path = input_path.join("preview.png");
        if !preview_path.is_file() {
            bail!("Couldn't find preview.png");
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
            println!(
                "✔️  Wrote original preview image in {}ms",
                now.elapsed().as_millis()
            );
        }

        [128u32, 256, 512, 1024].par_iter().for_each(|size| {
            let now = Instant::now();
            println!("▶️  Building x{} image", size);

            let thumb = img.thumbnail(*size, *size);
            let thumb_path = output_path.join(format!("preview_{}.png", size));

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
