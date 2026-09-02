FROM rust:1.85-bookworm AS builder

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY examples ./examples

# Library build (verify)
RUN cargo build --release --lib

# Demo builds (spacelab + server)
RUN cargo build --release --example spacelab
RUN cargo build --release --example server --features server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/target/release/examples/server ./server

EXPOSE 8080/udp

ENV RUST_LOG=info

CMD ["./server"]
