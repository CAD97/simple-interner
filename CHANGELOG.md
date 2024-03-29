# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
with the usual `0.MAJOR.MINOR` extension.

<!--
conventional sections:
### Added      | Public API additions
### Changed    | Changes to library behavior
### Deprecated | Public API deprecations
### Fixed      | Bug fixes
### Improved   | Internal improvements
### Removed    | Public API removals
### Security   | Security-related changes (e.g. soundness fixes)
-->

## [Unreleased]

## [0.3.4] - 2023-02-04

### Fixed

- *Actually* published the tagged git sha, hopefully. CI no likey publishing.

## [0.3.3] - 2023-02-04

### Fixed

- Fixed crates-io detection of README
- Actually published the git sha which is tagged... pending publish scripts

## [0.3.2] - 2023-02-04

Due to a mishap with publishing scripts, the published commit is a different
SHA than the release actually tagged in Git. This should be fixed by 0.3.3.

### Added

- Started keeping a changelog

### Improved

- Updated CI conventions
- Automated release publishing with cargo-release

<!-- and now for the comparison link urls: -->

[Unreleased]: https://github.com/cad97/simple-interner/compare/v0.3.4...HEAD
[0.3.4]: https://github.com/cad97/simple-interner/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/cad97/simple-interner/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/cad97/simple-interner/compare/v0.3.1...v0.3.2
