#!/usr/bin/env bash

cargo check
cargo clippy -- -D clippy::all
# pedantic is a bit much
#cargo clippy --all-features -- --deny warnings --deny clippy::pedantic --deny clippy::nursery
cargo clippy --all-features -- --deny warnings --deny clippy::nursery

cargo sqlx prepare --check --workspace
