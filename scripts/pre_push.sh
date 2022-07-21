#!/usr/bin/env bash

cargo check
cargo clippy -- -D clippy::all
# pedantic is a bit much
#cargo clippy --all-features -- --deny warnings --deny clippy::pedantic --deny clippy::nursery
cargo clippy --all-features -- --deny warnings --deny clippy::nursery

PROJECT_DIR=$(pwd)

cd $PROJECT_DIR/apps/server
cargo sqlx prepare \
    --check \
    -- --lib

cd $PROJECT_DIR/apps/exporter
cargo sqlx prepare \
    --check \
    -- --bin swu-exporter
