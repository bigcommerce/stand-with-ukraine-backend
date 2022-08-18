#!/usr/bin/env bash

export SQLX_OFFLINE=true
PROJECT_DIR=$(pwd)

cd $PROJECT_DIR/apps/server
cargo sqlx prepare \
    -- --lib

cd $PROJECT_DIR/apps/exporter
cargo sqlx prepare \
    -- --bin swu-exporter
