mod meh_data_commands;
mod clap_command;

use std::path::{Path};
pub use clap_command::ClapCommand;
pub use meh_data_commands::*;

pub trait MehDataCommand {
    fn get_description(&self) -> &str;
    fn exec(&self, input_path: &Path, output_path: &Path) -> anyhow::Result<()>;
}

pub struct DummyMehDataCommand {}
impl MehDataCommand for DummyMehDataCommand {
    fn get_description(&self) -> &str {
        "dummy"
    }

    fn exec(&self, _: &Path, _: &Path) -> anyhow::Result<()> {
        Ok(())
    }
}