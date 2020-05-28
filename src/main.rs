#![cfg(feature = "application")]
use color_eyre::Help;
use eyre::WrapErr;
use hypothesis::cli::HypothesisCLI;
use hypothesis::errors::CLIError;
use hypothesis::Hypothesis;
use structopt::StructOpt;

fn main() -> color_eyre::Result<()> {
    let cli: HypothesisCLI = HypothesisCLI::from_args();
    let api = Hypothesis::from_env()
        .wrap_err(CLIError::AuthorizationError)
        .suggestion("Make sure $HYPOTHESIS_NAME is set to your username and $HYPOTHESIS_KEY is set to your personal API key")?;
    cli.run(api)?;
    Ok(())
}
