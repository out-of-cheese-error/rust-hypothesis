## A Rust API for [Hypothesis](https://web.hypothes.is/)
### Work in progress

Based on the [Hypothesis API (1.0.0)](https://h.readthedocs.io/en/latest/api-reference/v1/).

Caveats:
- Only supports APIKey authorization and hypothes.is authority (i.e. single users)
- RangeSelector doesn't seem to follow [W3C standards](https://www.w3.org/TR/annotation-model/#range-selector). It's just a hashmap for now.
- Need to figure out how Document works to properly document it (hah).
- Can't delete a group after making it, can leave it though (maybe it's the same thing?)
- No idea what `UserProfile.preferences` and `UserProfile.features` mean.
 