# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.1](https://github.com/lzanini/mdbook-katex/compare/v0.9.0...v0.9.1) - 2024-11-07

### Fixed

- fix deploy CI artifact names

### Other

- make clippy happy
- switch to mdbook_fork4ls v0.4.41; update dependencies
- deploy CI explicit release tag
- different tag for deploy than release
- deploy CI does not publish on crates.io
- allow manually trigger deploy CI

## [0.9.0](https://github.com/lzanini/mdbook-katex/compare/v0.8.1...v0.9.0) - 2024-05-23

### Fixed
- fix&enhance tracing subscriber output

### Other
- update dependencies
- use tracing
- print render error&restore delimiter
- build release binary on release-plz
