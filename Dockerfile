# SERVE
FROM rust:latest as serve

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev

COPY serve/ /serve/
WORKDIR /serve
RUN cargo build --release --target x86_64-unknown-linux-musl

# BUILD STATIC
FROM rust:latest as builder

WORKDIR /build

# cache build if nothing changed but pages
COPY Cargo.toml Cargo.toml
COPY Cargo.toml Cargo.toml
COPY src/ src/
RUN cargo build --release

COPY site/ site/
RUN cargo run --release

# FNAL IMAGE
FROM alpine

WORKDIR /app

COPY --from=builder /build/dist/ /app/dist
COPY static/ static/

COPY --from=serve /serve/target/x86_64-unknown-linux-musl/release/serve /app/serve
COPY --from=serve /serve/Rocket.toml /app/

ENV SERVE_DIR /app
CMD ["./serve"]
