#!/usr/bin/env bash

cargo check
cargo clippy -- -D clippy::all
# pedantic is a bit much
#cargo clippy --all-features -- --deny warnings --deny clippy::pedantic --deny clippy::nursery
cargo clippy --all-features -- --deny warnings --deny clippy::nursery

cd app/
cargo sqlx prepare \
    --database-url "postgres://${POSTGRES_USER:=postgres}:${POSTGRES_PASSWORD:=password}@localhost:${POSTGRES_PORT:=5432}/${POSTGRES_DB:=swu}" \
    --check \
    -- --lib
cd ..

cd exporter/
cargo sqlx prepare \
    --database-url "postgres://${POSTGRES_USER:=postgres}:${POSTGRES_PASSWORD:=password}@localhost:${POSTGRES_PORT:=5432}/${POSTGRES_DB:=swu}" \
    --check \
    -- --bin swu-exporter
cd ..
