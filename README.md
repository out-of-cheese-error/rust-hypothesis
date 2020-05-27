# A Rust API for [Hypothesis](https://web.hypothes.is/)
### Work in progress

## Description
A lightweight wrapper for the [Hypothesis Web API v1.0.0](https://h.readthedocs.io/en/latest/api-reference/v1/). 
It includes helper functions for all APIKey authorized endpoints related to 
* annotations (create / update / delete / search / fetch / flag), 
* groups (create / update / list / fetch / leave / members) 
* profile (user information / groups)

## Getting Started
### Authorization
You'll need a [Hypothesis](https://hypothes.is) account, and a personal API token obtained as described [here](https://h.readthedocs.io/en/latest/api/authorization/). 
The code refers to your Hypothesis username as `username` and the API token as `developer_key`. 

### Examples
Check the [documentation](https://docs.rs/crate/hypothesis) of the `Hypothesis` struct for some usage examples.

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
 
