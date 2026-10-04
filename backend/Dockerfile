FROM rust:1.98-bookworm AS builder
WORKDIR /app/backend
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release --bin ybm-connect

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/backend/target/release/ybm-connect /usr/local/bin/ybm-connect
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV APP_ENV=production
EXPOSE 8080
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/ybm-connect"]