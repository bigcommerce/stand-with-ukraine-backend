#!/usr/bin/env bash
# set -x
set -o pipefail

if ! [ -x "$(command -v docker)" ]; then
	echo >&2 "Error: docker is not installed."
	exit 1
fi
if ! [ -x "$(command -v psql)" ]; then
	echo >&2 "Error: psql is not installed."
	exit 1
fi
if ! [ -x "$(command -v sqlx)" ]; then
	echo >&2 "Error: sqlx is not installed."
	echo >&2 "Use:"
	echo >&2 " cargo install --version=0.5.7 sqlx-cli --no-default-features --features postgres"
	echo >&2 "to install it."
	exit 1
fi

DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=swu}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_CONTAINER_NAME="swu-db"

# Allow to skip Docker if a dockerized Postgres database is already running
if [[ "${CREATE_LOCAL_DB}" == "TRUE" ]]; then
	# # Cleanup existing docker containers

	docker kill $DB_CONTAINER_NAME || true
	docker rm $DB_CONTAINER_NAME || true

	echo "Starting container ${DB_CONTAINER_NAME}"
	docker run \
		--name "${DB_CONTAINER_NAME}" \
		-e POSTGRES_USER=${DB_USER} \
		-e POSTGRES_PASSWORD=${DB_PASSWORD} \
		-e POSTGRES_DB=${DB_NAME} \
		-p "${DB_PORT}":5432 \
		-d postgres \
		postgres -N 1000 >/dev/null 2>&1
fi

# Keep pinging Postgres until it's ready to accept commands
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q' >/dev/null 2>&1; do
	echo >&2 "Postgres is still unavailable at localhost:${DB_PORT} - sleeping"
	sleep 1
done

echo >&2 "Postgres is up and running on port ${DB_PORT}!"
export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run
