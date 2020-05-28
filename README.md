[![Crates.io](https://img.shields.io/crates/v/hypothesis.svg)](https://crates.io/crates/hypothesis)
# A Rust API for [Hypothesis](https://web.hypothes.is/)
### Work in progress

## Description
A lightweight wrapper and CLI for the [Hypothesis Web API v1.0.0](https://h.readthedocs.io/en/latest/api-reference/v1/). 
It includes all APIKey authorized endpoints related to 
* annotations (create / update / delete / search / fetch / flag), 
* groups (create / update / list / fetch / leave / members) 
* profile (user information / groups)

## Installation and Usage
### Authorization
You'll need a [Hypothesis](https://hypothes.is) account, and a personal API token obtained as described [here](https://h.readthedocs.io/en/latest/api/authorization/). 
Set the environment variables `$HYPOTHESIS_NAME` and `$HYPOTHESIS_KEY` to your username and the developer API key respectively.

### As a command-line utility:
```bash
cargo install hypothesis
```
Run `hypothesis --help` to see subcommands and options.

Generate shell completions:
```bash
hypothesis complete zsh > .oh-my-zsh/completions/_hypothesis
exec zsh
```

### As a Rust crate
Add to your Cargo.toml:
```toml
[dependencies]
hypothesis = {version = "0.2.0", default-features = false}
```

#### Examples
Check the [documentation](https://docs.rs/crate/hypothesis) of the `Hypothesis` struct for some usage examples.
If you want to use environment variables, instantiate the api with `from_env` instead of `new`.

TODO: Add a longer example here

### Changelog
See the [CHANGELOG](CHANGELOG.md)

### Caveats / Todo:
- Blocking API (nothing stopping async except my lack of experience with it though).
- Only supports APIKey authorization and hypothes.is authority (i.e. single users).
- `Target.selector.RangeSelector` doesn't seem to follow [W3C standards](https://www.w3.org/TR/annotation-model/#range-selector). It's just a hashmap for now.
- Need to figure out how `Document` works to properly document it (hah).
- Can't delete a group after making it, can leave it though (maybe it's the same thing?)
- No idea what `UserProfile.preferences` and `UserProfile.features` mean.
- CLI just dumps output as JSON, this is fine right? Fancier CLIs can build on top of this (or use the crate directly)
