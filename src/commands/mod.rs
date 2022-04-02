mod mvt;
mod preview;
mod terrain_rgb;
mod sat;

use std::path::{Path, PathBuf};
use anyhow::{bail, Error, Result};
use clap::{App, arg, ArgMatches};
use geo::{Coordinate};
use mapbox_vector_tile::Layer;
use crate::commands::mvt::MapboxVectorTiles;
use crate::commands::preview::Preview;
use crate::commands::sat::Sat;
use crate::commands::terrain_rgb::TerrainRGB;
use crate::SerdeMetaJsonParser;



pub trait ClapCommand {
    fn get_identifier(&self) -> &str;
    fn register(&self) -> App;
    fn exec(&self, matches: &ArgMatches) -> Result<()>;
}

pub fn register_input_output_path_parameters(app: App) -> App {
    app
        .arg(arg!(-i --input <INPUT_DIR> "Path to grad_meh map directory"))
        .arg(arg!(-o --output <OUTPUT_DIR> "Path to output directory"))
}

pub fn get_input_output_path_parameters(matches: &ArgMatches) -> Result<(PathBuf, PathBuf)> {
    let input_path_str = matches.value_of("input").unwrap();
    let output_path_str = matches.value_of("output").unwrap();

    let (input_path, output_path) = (Path::new(input_path_str).to_path_buf(), Path::new(output_path_str).to_path_buf());

    if !output_path.is_dir() {
        bail!("Output path is not a directory");
    }

    if !input_path.is_dir() {
        bail!("Input path is not a directory");
    }

    Ok((input_path, output_path))
}

pub struct SatCommand {}
impl ClapCommand for SatCommand {
    fn get_identifier(&self) -> &str {
        "sat"
    }
    fn register(&self) -> App {
        let app = App::new(self.get_identifier()).about("Build satellite tiles from grad_meh data.");
        register_input_output_path_parameters(app)
    }

    fn exec(&self, matches: &ArgMatches) -> Result<(), Error> {
        let (input_path, output_path) = get_input_output_path_parameters(matches)?;
        (Sat {}).exec(&input_path, &output_path)
    }
}
pub struct MvtCommand {}
impl ClapCommand for MvtCommand {
    fn get_identifier(&self) -> &str {
        "mvt"
    }

    fn register(&self) -> App {
        let app = App::new(self.get_identifier()).about("Build mapbox vector tiles from grad_meh data.");
        register_input_output_path_parameters(app)
    }

    fn exec(&self, matches: &ArgMatches) -> Result<(), Error> {
        let (input_path, output_path) = get_input_output_path_parameters(matches)?;
        let mvt = MapboxVectorTiles::new(Box::new(SerdeMetaJsonParser {}));
        mvt.exec(&input_path, &output_path)
    }
}

pub struct MvtTestCommand {}
impl ClapCommand for MvtTestCommand {
    fn get_identifier(&self) -> &str {
        "mvt_test"
    }

    fn register(&self) -> App {
        App::new(self.get_identifier()).about("test mapbox_vector_tiles crate that I want to use")
    }

    fn exec(&self, _: &ArgMatches) -> Result<()> {
        let mut tile = mapbox_vector_tile::Tile::new();
        tile.add_layer("foo_layer");
        tile.add_feature("foo_layer", mapbox_vector_tile::Feature::from(geo::Geometry::Point(geo::Point(Coordinate { x: 1, y: 1}))));
        tile.add_feature("foo_layer", mapbox_vector_tile::Feature::from(geo::Geometry::Polygon(geo::Polygon::new(geo::LineString(vec![
            Coordinate {x: 0, y: 0},
            Coordinate {x: 0, y: 2},
            Coordinate {x: 2, y: 2},
            Coordinate {x: 2, y: 0},
        ]), vec![]))));
        tile.add_layer(Layer::new("bar"));
        tile.write_to_file("./foo.bar");

        Ok(())
    }
}

pub struct PreviewCommand {}
impl ClapCommand for PreviewCommand {
    fn get_identifier(&self) -> &str {
        "preview"
    }

    fn register(&self) -> App {
        let app = App::new(self.get_identifier()).about("Build resolutions for preview image.");
        register_input_output_path_parameters(app)
    }

    fn exec(&self, matches: &ArgMatches) -> Result<(), Error> {
        let (input_path, output_path) = get_input_output_path_parameters(matches)?;
        (Preview {}).exec(&input_path, &output_path)
    }
}
pub struct TerrainRgbCommand {}
impl ClapCommand for TerrainRgbCommand {
    fn get_identifier(&self) -> &str {
        "terrain_rgb"
    }

    fn register(&self) -> App {
        let app = App::new(self.get_identifier()).about("Build Terrain-RGB tiles from grad_meh data.");
        register_input_output_path_parameters(app)
    }

    fn exec(&self, matches: &ArgMatches) -> Result<(), Error> {
        let (input_path, output_path) = get_input_output_path_parameters(matches)?;
        (TerrainRGB {}).exec(&input_path, &output_path)
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use std::path::PathBuf;
    use clap::{App, ArgMatches};
    use crate::commands::{get_input_output_path_parameters, register_input_output_path_parameters};
    use crate::test::with_input_and_output_paths;

    fn clap_command_with_params(args: Vec<String>) -> anyhow::Result<(PathBuf, PathBuf)> {
        let app = register_input_output_path_parameters(App::new("x"));
        let matches: ArgMatches = app.get_matches_from(args);

        get_input_output_path_parameters(&matches)
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
}
