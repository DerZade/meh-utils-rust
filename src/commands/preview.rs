use clap::{arg, App};

use crate::commands::{ Command };

pub struct Preview {}

impl Command for Preview {
    fn register(&self) -> App<'static> {
        App::new("preview")
            .about("Build resolutions for preview image.")
            .arg(arg!(-i --input <INPUT_DIR> "Path to grad_meh map directory"))
            .arg(arg!(-o --output <OUTPUT_DIR> "Path to output directory"))
    }
    fn run(&self, _args: &clap::ArgMatches) -> Result<(), &'static str> {
        unimplemented!();
    }
}