use clap::{App, arg};
use std::path::{Path, PathBuf};
use crate::MehDataCommand;

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
