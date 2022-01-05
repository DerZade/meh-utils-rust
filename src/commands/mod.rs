mod preview;
mod sat;

pub use preview::Preview;
pub use sat::Sat;

pub trait Command {
    fn register(&self) -> clap::App<'static>;
    fn run(&self, _args: &clap::ArgMatches) -> anyhow::Result<()> {
        unimplemented!();
    }
}
