# Stand With Ukraine App


[![codecov](https://codecov.io/gh/swu-bc/stand-with-ukraine-backend/branch/main/graph/badge.svg?token=6EN9JQRHPQ)](https://codecov.io/gh/swu-bc/stand-with-ukraine-backend)
![ci](https://github.com/swu-bc/stand-with-ukraine-backend/actions/workflows/general.yaml/badge.svg)

This repo contains the backend code for this BigCommerce marketplace app.
The backend is powered by a rust application built using `actix` (HTTP server) and `sqlx` (Database Library Postgres)

## Run locally

1. Install dependencies using `cargo install`
2. Initialize database using `./scripts/init_db.sh`
