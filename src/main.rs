#[cfg(not(feature = "cli"))]
fn main() {}

#[cfg(feature = "cli")]
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    use clap::Parser;
    use color_eyre::Help;
    use eyre::WrapErr;
    use hypothesis::cli::HypothesisCLI;
    use hypothesis::errors::CLIError;
    use hypothesis::Hypothesis;
    color_eyre::install()?;
    let cli: HypothesisCLI = HypothesisCLI::parse();
    let api = Hypothesis::from_env()
        .wrap_err(CLIError::AuthorizationError)
        .suggestion("Make sure $HYPOTHESIS_NAME is set to your username and $HYPOTHESIS_KEY is set to your personal API key")?;
    cli.run(api).await?;
    Ok(())
}
