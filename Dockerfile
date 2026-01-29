FROM rust:bookworm AS builder

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos

WORKDIR /app
COPY . .
RUN cargo leptos build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libsqlite3-0 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/soulcrush /usr/local/bin/
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/migrations /app/migrations

WORKDIR /app
ENV LEPTOS_SITE_ROOT=site
ENV LEPTOS_SITE_ADDR=0.0.0.0:8080
ENV DATABASE_URL=sqlite:/app/data/data.db?mode=rwc

VOLUME /app/data
EXPOSE 8080

CMD ["soulcrush"]
