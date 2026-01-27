FROM rust:bookworm AS builder

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos

WORKDIR /app
COPY . .
RUN cargo leptos build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/soulcrush /usr/local/bin/
COPY --from=builder /app/target/site /app/site

WORKDIR /app
ENV LEPTOS_SITE_ROOT=/app/site

CMD ["soulcrush"]
