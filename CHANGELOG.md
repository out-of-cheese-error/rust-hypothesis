# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Master
* everything is asynchronous.
* added a bulk API for modifying many things at once.
* Annotation tags are optional, to account for no tags vs single empty tag
* AnnotationMaker tags are optional to allow for removing a tag during update

## [0.2.0] - 2020-05-28
Works both as a crate and a binary now!
### Added
* a CLI version under a feature flag "cli"
* Better documentation
### Changed
License to BSD-2-Clause (consistent with hypothesis/h)

## 0.1.0 - 2020-05-27
First version! Complete API for annotations, groups and profile. 
Some missing docs and weird edges, listed in Caveats/Todos in the README.


[0.2.0]: https://github.com/out-of-cheese-error/the-way/releases/tag/v0.2.0