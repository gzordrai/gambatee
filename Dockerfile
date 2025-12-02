FROM rust:1.91.1-alpine AS builder

RUN apk add --no-cache \
    build-base \
    ca-certificates \
    openssl-dev \
    openssl-libs-static \
    musl-dev \
    pkgconfig

WORKDIR /app

COPY . .

RUN cargo build --release --no-default-features

FROM alpine:latest

RUN apk add --no-cache \
    ca-certificates \
    openssl

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/gambatee .
COPY --from=builder /app/config.toml .

ENTRYPOINT ["/usr/local/bin/gambatee"]
