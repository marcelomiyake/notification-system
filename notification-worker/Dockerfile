FROM rust:1.88-bookworm AS builder
WORKDIR /src
COPY notification-worker/Cargo.toml notification-worker/Cargo.toml
COPY notification-worker/Cargo.lock notification-worker/Cargo.lock
COPY notification-worker/src notification-worker/src
COPY database/migrations database/migrations
WORKDIR /src/notification-worker
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/notification-worker/target/release/notification-worker /usr/local/bin/notification-worker
USER 10001:10001
EXPOSE 8081
ENTRYPOINT ["/usr/local/bin/notification-worker"]
