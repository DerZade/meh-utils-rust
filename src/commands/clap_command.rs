use clap::{App, arg};
use std::path::{Path, PathBuf};
use anyhow::bail;
use crate::MehDataCommand;

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use clap::{ArgMatches};
    use crate::ClapCommand;
    use crate::commands::DummyMehDataCommand;
    use crate::utils::with_input_and_output_paths;

    fn clap_command_with_params(args: Vec<String>) -> anyhow::Result<()> {
        let cmd = ClapCommand { identifier: "foo".to_string(), exec:  &DummyMehDataCommand {} };
        let app = cmd.register();


        let matches: ArgMatches = app.get_matches_from(args);

        cmd.run(&matches)
    }

    #[test]
    fn clap_command_will_bail_on_input_path_not_existing() {
        with_input_and_output_paths(|input_path, _| {
            let input_path_str = match input_path.to_str() {Some(s) => s, None => ""}.to_string();
            let res = clap_command_with_params(vec![
                "/foo/pars".to_string(),
                "--input".to_string(),
                input_path_str,
                "--output".to_string(),
                "/bar/baz".to_string()
            ]);

            assert!(res.is_err());

            ()
        });
    }

    #[test]
    fn clap_command_will_bail_on_output_path_not_existing() {
        with_input_and_output_paths(|_, output_path| {
            let output_path_str = match output_path.to_str() {Some(s) => s, None => ""}.to_string();
            let res = clap_command_with_params(vec![
                "/foo/pars".to_string(),
                "--input".to_string(),
                "/bar/baz".to_string(),
                "--output".to_string(),
                output_path_str,
            ]);

            assert!(res.is_err());

            ()
        });
    }

    #[test]
    fn clap_command_new_builds_correctly() {
        let cmd = ClapCommand::new("foo", &DummyMehDataCommand {});
        assert_eq!(cmd.identifier, "foo".to_string());
        assert_eq!(cmd.exec.get_description(), "dummy");

    }
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

        if !output_path.is_dir() {
            bail!("Output path is not a directory");
        }

        if !input_path.is_dir() {
            bail!("Input path is not a directory");
        }

        self.exec.exec(&input_path, &output_path)
    }

    fn get_in_out_path_params(&self, args: &clap::ArgMatches) -> (PathBuf, PathBuf) {
        let input_path_str = args.value_of("input").unwrap();
        let output_path_str = args.value_of("output").unwrap();

        (Path::new(input_path_str).to_path_buf(), Path::new(output_path_str).to_path_buf())
    }
}
