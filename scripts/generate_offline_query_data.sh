#!/usr/bin/env bash

export SQLX_OFFLINE=true
cargo sqlx prepare --workspace
