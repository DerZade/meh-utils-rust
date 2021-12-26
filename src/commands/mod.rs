mod preview;

pub use preview::Preview;

pub trait Command {
    fn register(&self) -> clap::App<'static>;
    fn run(&self, _args: &clap::ArgMatches) -> Result<(), &'static str> {
        unimplemented!();
    }
}