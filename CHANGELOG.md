# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.0](https://github.com/lzanini/mdbook-katex/compare/v0.9.4...v0.10.0) - 2025-11-28

### Other

- update dependencies: toml to v0.9, rayon to 1.11
- use mdbook-preprocessor v0.5.1 instead of mdbook_fork4ls (a fork of mdbook v0.4, [#131](https://github.com/lzanini/mdbook-katex/pull/131))
- allow use of different js renderers of katex through features `quick-js` and `duktape`
- changed default features to `quick-js`

## [0.9.4](https://github.com/lzanini/mdbook-katex/compare/v0.9.3...v0.9.4) - 2025-05-01

### Other

- make clippy happy;update dependencies
- use mdbook_fork4ls v0.4.48

## [0.9.3](https://github.com/lzanini/mdbook-katex/compare/v0.9.2...v0.9.3) - 2025-02-18

### Other

- use mdbook_fork4ls v0.4.45 & its builtin parallelization

## [0.9.2](https://github.com/lzanini/mdbook-katex/compare/v0.9.1...v0.9.2) - 2024-12-07

### Other

- make new clippy happy
- update dependency mdbook to v0.4.43;update other dependencies
- attempt to fix upload-artifact not allowing reused name
- bump download-artifact as well
- bump upload-artifact to v4 so deployment runs

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
