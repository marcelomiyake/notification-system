FROM rust:1.88-bookworm AS builder
WORKDIR /src
COPY notification-api/Cargo.toml notification-api/Cargo.toml
COPY notification-api/Cargo.lock notification-api/Cargo.lock
COPY notification-api/src notification-api/src
COPY database/migrations database/migrations
WORKDIR /src/notification-api
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/notification-api/target/release/notification-api /usr/local/bin/notification-api
USER 10001:10001
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/notification-api"]
