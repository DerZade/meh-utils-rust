use clap::{app_from_crate, AppSettings};
use crate::commands::{ClapCommand, MehDataCommand, Preview, Sat, TerrainRGB, MapboxVectorTiles};
use crate::metajson::SerdeMetaJsonParser;


mod commands;
mod dem;
mod metajson;
mod tilejson;
mod utils;
mod mvt;
mod feature;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if let Err(e) = execute(&args) {
        println!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

fn execute(input: &[String]) -> anyhow::Result<()> {
    let commands: Vec<ClapCommand> = vec![
        ClapCommand::new("preview", Box::new(Preview {})),
        ClapCommand::new("sat", Box::new(Sat {})),
        ClapCommand::new("terrain_rgb", Box::new(TerrainRGB {})),
        ClapCommand::new("mvt", Box::new(MapboxVectorTiles::new(Box::new(SerdeMetaJsonParser {})))),
        // Add commands here
    ];

    let mut app = app_from_crate!()
        .global_setting(AppSettings::PropagateVersion)
        .global_setting(AppSettings::UseLongFormatForHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp);

    app = commands.iter().fold(app, |a, c| a.subcommand(c.register()));

    let matches = app.get_matches_from(input);

    let result = match matches.subcommand() {
        Some((name, sub_matches)) => match commands.iter().filter(|c| {c.identifier == name}).next() {
            Some(command) => command.run(sub_matches),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    result
}
