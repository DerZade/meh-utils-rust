mod meh_data_commands;
mod clap_command;

use std::path::{Path};
pub use clap_command::ClapCommand;
pub use meh_data_commands::*;

pub trait MehDataCommand {
    fn get_description(&self) -> &str;
    fn exec(&self, input_path: &Path, output_path: &Path) -> anyhow::Result<()>;
}