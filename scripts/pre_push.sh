#!/usr/bin/env bash

cargo check
cargo clippy -- -D clippy::all

cargo sqlx prepare \
    --database-url "postgres://${POSTGRES_USER:=postgres}:${POSTGRES_PASSWORD:=password}@localhost:${POSTGRES_PORT:=5432}/${POSTGRES_DB:=swu}" \
    --check \
    -- --lib
