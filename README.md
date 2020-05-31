<!-- cargo-sync-readme start -->

[![Crates.io](https://img.shields.io/crates/v/hypothesis.svg)](https://crates.io/crates/hypothesis)
# A Rust API for [Hypothesis](https://web.hypothes.is/)

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
hypothesis = {version = "0.3.0", default-features = false}
# For a tokio runtime:
tokio = { version = "0.2", features = ["macros"] }
```

#### Examples
```rust
use hypothesis::Hypothesis;
use hypothesis::annotations::{AnnotationMaker, Target, Selector, TextQuoteSelector};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
   let api = Hypothesis::from_env()?;
   let new_annotation = api.create_annotation(
        &AnnotationMaker {
            uri: "https://www.example.com".to_owned(),
           text: "this is a comment".to_owned(),
           target: Target {
               source: "https://www.example.com".to_owned(),
               selector: vec![Selector::new_quote("exact text in website to highlight",
                                                  "prefix of text",
                                                  "suffix of text")],
           },
           tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
           .. Default::default()
       }
   ).await?;
   Ok(())
}
```
Use bulk functions to perform multiple actions - e.g. `api.fetch_annotations` instead of a
loop around `api.fetch_annotation`.

Check the [documentation](https://docs.rs/crate/hypothesis) for more usage examples.

### Changelog
See the [CHANGELOG](CHANGELOG.md)

### Caveats / Todo:
- ~~Blocking API (nothing stopping async except my lack of experience with it though)~~ Async from v0.3!.
- Only supports APIKey authorization and hypothes.is authority (i.e. single users).
- `Target.selector.RangeSelector` doesn't seem to follow [W3C standards](https://www.w3.org/TR/annotation-model/#range-selector). It's just a hashmap for now.
- `Annotation` hypermedia links are stored as a hashmap, b/c I don't know all the possible values.
- Need to figure out how `Document` works to properly document it (hah).
- Can't delete a group after making it, can leave it though (maybe it's the same thing?)
- No idea what `UserProfile.preferences` and `UserProfile.features` mean.
- CLI just dumps output as JSON, this is fine right? Fancier CLIs can build on top of this (or use the crate directly)

<!-- cargo-sync-readme end -->