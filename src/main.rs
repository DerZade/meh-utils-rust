use clap::{App, app_from_crate, AppSettings};
use crate::commands::{ClapCommand, PreviewCommand, SatCommand, MvtCommand, TerrainRgbCommand, MvtTestCommand};
use crate::metajson::SerdeMetaJsonParser;


mod commands;
mod dem;
mod metajson;
mod tilejson;
mod utils;
mod mvt;
mod feature;
#[cfg(test)]
mod test;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if let Err(e) = execute(&args) {
        println!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

fn execute(input: &[String]) -> anyhow::Result<()> {
    let commands: Vec<&dyn ClapCommand> = vec![
        &PreviewCommand {},
        &SatCommand {},
        &TerrainRgbCommand {},
        &MvtCommand {},
        &MvtTestCommand {},
        // Add commands here
    ];

    let mut app = app_from_crate!()
        .global_setting(AppSettings::PropagateVersion)
        .global_setting(AppSettings::UseLongFormatForHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp);

    app = commands.iter().fold(app, |main_app: App, subcommand| {
        main_app.subcommand(subcommand.register())
    });

    let matches = app.get_matches_from(input);

    let result = match matches.subcommand() {
        Some((name, sub_matches)) => match commands.iter().filter(|c| {c.get_identifier() == name}).next() {
            Some(command) => command.exec(sub_matches),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    result
}
