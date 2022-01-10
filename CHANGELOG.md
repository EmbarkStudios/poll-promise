# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `web` feature to enable `Promise::spawn_async` using [`wasm_bindgen_futures::spawn_local`](https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen_futures/fn.spawn_local.html).


## [0.1.0] - 2022-01-10
### Added
- Initial commit - add the `Promise` type.

[Unreleased]: https://github.com/EmbarkStudios/poll-promise/compare/0.1.1...HEAD
[0.1.0]: https://github.com/EmbarkStudios/poll-promise/releases/tag/0.1.0
