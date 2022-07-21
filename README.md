# Stand With Ukraine App

[![codecov](https://codecov.io/gh/swu-bc/stand-with-ukraine-backend/branch/main/graph/badge.svg?token=6EN9JQRHPQ)](https://codecov.io/gh/swu-bc/stand-with-ukraine-backend)
[![server](https://github.com/swu-bc/stand-with-ukraine-backend/actions/workflows/server.yaml/badge.svg)](https://github.com/swu-bc/stand-with-ukraine-backend/actions/workflows/server.yaml)
[![exporter](https://github.com/swu-bc/stand-with-ukraine-backend/actions/workflows/exporter.yaml/badge.svg)](https://github.com/swu-bc/stand-with-ukraine-backend/actions/workflows/exporter.yaml)

This repo contains the backend code for this BigCommerce marketplace app.
The backend is powered by a rust application built using `actix` (HTTP server) and `sqlx` (Database Library Postgres)

## Run locally

- Prerequisites
  - Rust toolchain
    - Recommend using `rustup` to setup `rust`, `cargo`, `fmt`
  - SQLX command
    - Recommend setup using `cargo install sqlx-cli --force --version=0.5.11 --features=postgres,rustls --no-default-features`
  - Docker
    - Recommended setup for `macos` or `linux` is `podman` and creating an alias for docker from podman
  - Editor
    - Recommended setup is `vscode` and the `rust-analyzer` extension.
  - For parsing logging `bunyan` command is helpful.
    - Recommended setup is `cargo install bunyan`
    - Enable log during testing and pass it through bunyan `TEST_LOG=true cargo test | bunyan`

1. Install dependencies using `cargo install`
2. Initialize database using `CREATE_LOCAL_DB=TRUE ./scripts/init_db.sh`

## Deployment

The app is deployed to Google cloud run using a Docker container. Github Actions tests and builds a docker image if everything passes.
The action is also responsible for pushing the image to google cloud artifcats and also running the update deployment command to target the new image. The production app is connected to a Cloud SQL managed Postgres Database.

## Architecture - API/Routes

- BigCommerce OAuth Routes. They are responsible for handling the install, load and uninstall requests from a BigCommerce Store
  - `/bigcommerce/install`
  - `/bigcommerce/load`
  - `/bigcommerce/uninstall`
- API Routes
  - `/api/v1/publish`
    - `POST` publish widget to storefront
    - `DELETE` remove widget from storefront
  - `/api/v1/preview`
    - `GET` retrieve the store url for previewing the widget
  - `/api/v1/configuration`
    - `POST` set the configuration of the widget
    - `GET` get the current configuration of the widget


## License

Copyright (c) 2017-present, BigCommerce Pty. Ltd. All rights reserved

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the Software without restriction, including without limitation the
rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit
persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the
Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE
WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.