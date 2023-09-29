# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]


## [0.2.1] - 2023-09-29
### Fixed
- Undefined behavior in `PromiseImpl::poll` and `PromiseImpl::block_until_ready`


## [0.2.0] - 2022-10-25
### Added
- `web` feature to enable `Promise::spawn_async` using [`wasm_bindgen_futures::spawn_local`](https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen_futures/fn.spawn_local.html).
- Add `Promise::abort` to abort the associated async task, if any.


## [0.1.0] - 2022-01-10
### Added
- Initial commit - add the `Promise` type.

[Unreleased]: https://github.com/EmbarkStudios/poll-promise/compare/0.2.0...HEAD
[0.2.1]: https://github.com/EmbarkStudios/poll-promise/releases/tag/0.2.1
[0.2.0]: https://github.com/EmbarkStudios/poll-promise/releases/tag/0.2.0
[0.1.0]: https://github.com/EmbarkStudios/poll-promise/releases/tag/0.1.0
