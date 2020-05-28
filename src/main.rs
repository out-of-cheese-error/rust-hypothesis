#![cfg(feature = "application")]
use hypothesis::cli::HypothesisCLI;
use structopt::StructOpt;

fn main() -> color_eyre::Result<()> {
    let cli = HypothesisCLI::from_args();
    Ok(())
}
