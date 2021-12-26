use std::{collections::HashMap};
use clap::{app_from_crate, AppSettings};

mod commands;

fn main() {
    let args: Vec<_> = std::env::args().collect();


    if let Err(e) = execute(&args) {
        println!("{}", e);
        std::process::exit(1);
    }
}

fn execute(input: &[String]) -> Result<(), Box<dyn std::error::Error>> {

    let mut app = app_from_crate!()
        .global_setting(AppSettings::PropagateVersion)
        .global_setting(AppSettings::UseLongFormatForHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp);
   
    let mut commands_by_name: HashMap<String, &Box<dyn commands::Command>> = HashMap::new();
    let mut commands: Vec<Box<dyn commands::Command>> = Vec::new();

    // Add commands here
    commands.push(Box::new(commands::Preview {}));

    for command in commands.iter() {
        let sub = command.register();
        commands_by_name.insert(sub.get_name().to_owned(), command);
        app = app.subcommand(sub);
    }

    let matches = app.get_matches_from(input);

    let result = match matches.subcommand() {
        Some((name, sub_matches)) => match commands_by_name.get(name) {
            Some(command) => {
                command.run(sub_matches)
            }
            _ => unreachable!()
        },
        _ => unreachable!(),
    };

    result
}