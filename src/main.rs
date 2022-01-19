use clap::{app_from_crate, AppSettings};
use std::collections::HashMap;
use crate::commands::{ClapCommand, MehDataCommand, Preview, Sat, TerrainRGB, MapboxVectorTiles};



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
    let mut app = app_from_crate!()
        .global_setting(AppSettings::PropagateVersion)
        .global_setting(AppSettings::UseLongFormatForHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp);

    let mut commands_by_name: HashMap<String, &ClapCommand> = HashMap::new();
    let commands: Vec<ClapCommand> = vec![
        ClapCommand::new("preview", &Preview {}),
        ClapCommand::new("sat", &Sat {}),
        ClapCommand::new("terrain_rgb", &TerrainRGB {}),
        ClapCommand::new("mvt", &MapboxVectorTiles {}),
        // Add commands here
    ];

    for command in commands.iter() {
        let sub = command.register();
        commands_by_name.insert(sub.get_name().to_owned(), command);
        app = app.subcommand(sub);
    }

    let matches = app.get_matches_from(input);

    let result = match matches.subcommand() {
        Some((name, sub_matches)) => match commands_by_name.get(name) {
            Some(command) => command.run(sub_matches),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    result
}
