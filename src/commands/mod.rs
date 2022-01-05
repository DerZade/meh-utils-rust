mod preview;
mod sat;
mod terrain_rgb;

pub use preview::Preview;
pub use sat::Sat;
pub use terrain_rgb::TerrainRGB;

pub trait Command {
    fn register(&self) -> clap::App<'static>;
    fn run(&self, _args: &clap::ArgMatches) -> anyhow::Result<()> {
        unimplemented!();
    }
}
