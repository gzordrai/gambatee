FROM rust:1.90.0-slim AS builder

WORKDIR /app

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/gambatee .
COPY --from=builder /app/config.toml .

ENTRYPOINT ["/usr/local/bin/gambatee"]