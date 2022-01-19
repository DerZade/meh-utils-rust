mod mvt;
mod preview;
mod sat;
mod terrain_rgb;

use std::path::{Path, PathBuf};
use clap::{App, arg};
pub use self::mvt::MapboxVectorTiles;
pub use sat::Sat;
pub use terrain_rgb::TerrainRGB;
pub use preview::Preview;

pub trait MehDataCommand {
    fn get_description(&self) -> &str;
    fn exec(&self, input_path: &Path, output_path: &Path) -> anyhow::Result<()>;
}

pub struct ClapCommand {
    pub identifier: String,
    pub exec: &'static dyn MehDataCommand,
}

impl ClapCommand {
    pub fn new(identifier: &str, exec: &'static impl MehDataCommand) -> Self {
        ClapCommand { identifier: identifier.to_string(), exec }
    }

    pub fn register(&self) -> App<'static> {

        let app = App::new(&self.identifier)
            .about(self.exec.get_description().as_ref());

        app
            .arg(arg!(-i --input <INPUT_DIR> "Path to grad_meh map directory"))
            .arg(arg!(-o --output <OUTPUT_DIR> "Path to output directory"))
    }

    pub fn run(&self, args: &clap::ArgMatches) -> anyhow::Result<()> {
        let (input_path, output_path) = self.get_in_out_path_params(args);

        self.exec.exec(&input_path, &output_path)
    }

    fn get_in_out_path_params(&self, args: &clap::ArgMatches) -> (PathBuf, PathBuf) {
        let input_path_str = args.value_of("input").unwrap();
        let output_path_str = args.value_of("output").unwrap();

        (Path::new(input_path_str).to_path_buf(), Path::new(output_path_str).to_path_buf())
    }
}