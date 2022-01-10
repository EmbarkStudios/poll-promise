#!/bin/bash
script_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$script_path"
set -eux

# Checks all tests, lints etc.
# Basically does what the CI does.

cargo check --workspace --all-targets --features "tokio"
cargo check --target wasm32-unknown-unknown --features "web"
cargo test --workspace --doc --features "tokio"
cargo test --workspace --all-targets --features "tokio"
cargo clippy --workspace --all-targets --features "tokio" -- -D warnings -W clippy::all
cargo fmt --all -- --check

cargo doc --no-deps --features "tokio"
# cargo doc --target wasm32-unknown-unknown --no-deps --features "web"
