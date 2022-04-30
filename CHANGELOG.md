# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.10.3 - 2022-04-30
Updated rust edition, dependencies, lints

## 0.10.2 - 2021-04-13
### Changed
Added serde error and raw text to `APIError` for easier debugging

## 0.10.0 - 2021-04-13

### Fixed
Added all [w3 selectors](https://www.w3.org/TR/annotation-model/#selectors) to `Selector` enum

## 0.9.1 - 2021-03-26

Updated dependencies

## 0.9.0 - 2021-03-26

Added `Document` to `Annotation` output struct

## 0.8.0 - 2021-01-16

Updated dependencies

## 0.7.2 - 2020-11-28

Fixed typo: wildcard-uri -> wildcard_uri

## 0.7.1 - 2020-09-03

Added `builder` methods to generate Builders (https://matklad.github.io/2020/08/12/who-builds-the-builder.html)

## 0.7.0 - 2020-09-03

Added `search_annotations_return_all` which uses a loop to bypass the limit for number of annotations returned

## 0.6.0 - 2020-06-23
### Changed
* Library exposes `HypothesisError` instead of using `eyre`
### Fixed
* File creation bug (expected file to exist)
* `display_name` can be null now

## 0.5.0 - 2020-06-06
### Changed
* Update takes Annotation as input instead of InputAnnotation
* SearchQuery takes String as input, i.e. input needs to already be formatted as acct:{username}@hypothes.is
### Added
* Happy path tests for `annotations` and `groups` CLI 

## 0.4.0 - 2020-06-02
### Changed
* Switched `AnnotationID` and `GroupID` back to Strings and &str 
* Renamed `AnnotationMaker` to `InputAnnotation`

### Added
* made Builders for `InputAnnotation`, `Target, Document`, and `SearchQuery` using `derive_builder`
* better docs

## 0.3.0 - 2020-05-31
* everything is asynchronous.
* added a bulk API for modifying many things at once.
* `AnnotationMaker` tags are optional to allow for removing a tag during update

## 0.2.0 - 2020-05-28
Works both as a crate and a binary now!
### Added
* a CLI version under a feature flag "cli"
* Better documentation
### Changed
License to BSD-2-Clause (consistent with hypothesis/h)

## 0.1.0 - 2020-05-27
First version! Complete API for annotations, groups and profile. 
Some missing docs and weird edges, listed in Caveats/Todos in the README.