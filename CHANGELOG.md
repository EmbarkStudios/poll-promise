# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]
- Disable all blocking functions on Wasm, since you are not allowed to block in a browser


## [0.3.0] - 2023-08-27
### Added
- Support `async-std` executor [#15](https://github.com/EmbarkStudios/poll-promise/pull/15)
- `smol` feature to enable the use of [`smol`](https://github.com/smol-rs/smol)
- refactor `Promise::spawn_async` into two new functions, `Promise::spawn_async` and `Promise::spawn_local`
- `smol_tick_poll` feature to automatically tick the smol executor when polling promises

### Changed
- `spawn_async` is now called `spawn_local` on web.


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


[Unreleased]: https://github.com/EmbarkStudios/poll-promise/compare/0.3.0...HEAD
[0.3.0]: https://github.com/EmbarkStudios/poll-promise/releases/tag/0.3.0
[0.2.1]: https://github.com/EmbarkStudios/poll-promise/releases/tag/0.2.1
[0.2.0]: https://github.com/EmbarkStudios/poll-promise/releases/tag/0.2.0
[0.1.0]: https://github.com/EmbarkStudios/poll-promise/releases/tag/0.1.0
