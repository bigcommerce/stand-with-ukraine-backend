FROM lukemathwalker/cargo-chef:latest-rust-1.62-bullseye as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
# Build our project
RUN cargo build -p swu-app --release --bin swu-app

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
# Setup executable and configuration
COPY --from=builder /app/target/release/swu-app swu-app
COPY apps/server/configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./swu-app"]