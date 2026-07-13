# Changelog

All notable changes to the cotis-web workspace are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

Publishable workspace crates (`cotis-web`, `cotis-web-builder`) are released
together at the same version.

## [Unreleased]

## [0.1.0-alpha] - 2026-07-11

First public alpha of the cotis-web workspace.

### Added

- `cotis-web` — browser/WASM render backend implementing `CotisRendererAsync` for HTML/CSS output
- `cotis-web-builder` — `wasm-pack` build routine and `cotis-cli` plugin
- `web-example` — full WASM demo application
- CI workflow (format, clippy, test on Linux; wasm32 check/clippy/doc; tests on Windows)
- Open-source documentation: README, LICENSE, CONTRIBUTING, SECURITY, CHANGELOG

### Changed

- Reset version from the internal `0.1.0` line to `0.1.0-alpha` for public release
- Switched Cotis ecosystem dependencies from private registry to crates.io `0.1.0-alpha`
- Renamed feature `app_lauch` to `app_launch` (aligns with `cotis`)

### Features

- `cotis-web`: `app_launch`, `complex_color`, `complex_colored_text`

[Unreleased]: https://github.com/igna-778/cotis-web/compare/v0.1.0-alpha...HEAD
[0.1.0-alpha]: https://github.com/igna-778/cotis-web/releases/tag/v0.1.0-alpha
