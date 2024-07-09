#!/usr/bin/env bash

set -o pipefail
shopt -s expand_aliases

if [ "${USE_NERDCTL}" == "TRUE" ]; then
    alias container_cmd="nerdctl"
elif [ "${USE_PODMAN}" == "TRUE" ]; then
    alias container_cmd="podman"
else
    alias container_cmd="docker"
    if ! [ -x "$(command -v docker)" ]; then
        echo >&2 "Error: docker is not installed."
        exit 1
    fi
fi
if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed."
    exit 1
fi
if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo >&2 "Use:"
    echo >&2 " cargo install --version=0.7.4 sqlx-cli --features postgres,runtime-tokio,tls-rustls"
    echo >&2 "to install it."
    exit 1
fi

if [ -f .env ]; then
    set -o allexport
    source .env
    set +o allexport
fi

DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=swu}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_CONTAINER_NAME="swu-db"
DB_HOST="${POSTGRES_HOST:=localhost}"

# Allow to skip Docker if a dockerized Postgres database is already running
if [[ "${CREATE_LOCAL_DB}" == "TRUE" ]]; then
    # # Cleanup existing docker containers
    container_cmd kill $DB_CONTAINER_NAME || true
    container_cmd rm $DB_CONTAINER_NAME || true

    echo "Starting container ${DB_CONTAINER_NAME}"
    container_cmd run \
        --name "${DB_CONTAINER_NAME}" \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        -d postgres \
        postgres -N 1000 >/dev/null 2>&1
fi

# Keep pinging Postgres until it's ready to accept commands
COUNTER=0
COUNTER_LIMIT=10
export PGPASSWORD=${DB_PASSWORD}
until psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "${DB_NAME}" -c '\q' >/dev/null 2>&1; do
    if ((COUNTER >= COUNTER_LIMIT)); then
        ## Exit early because database is not online
        echo >&2 "Postgres has not come online after waiting for $COUNTER seconds"
        exit 999
    fi

    COUNTER=$((COUNTER + 1))
    echo >&2 "Waiting for postgres at ${DB_HOST}:${DB_PORT}"
    sleep 1
done

echo >&2 "Postgres is up and running on port ${DB_PORT}!"
DATABASE_URL_WITHOUT_DB=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}

# Create database with user name - required for e2e tests
psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "${DB_NAME}" -c "CREATE DATABASE ${DB_USER};" >/dev/null 2>&1
export DATABASE_URL=${DATABASE_URL_WITHOUT_DB}/${DB_NAME}
sqlx database create
sqlx migrate run
