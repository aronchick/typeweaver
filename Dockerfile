FROM rust:1.82-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release -p typeweaver-cli

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/typeweaver-cli /usr/local/bin/typeweaver

ENV TYPEWEAVER_REGISTRY_ROOT=/data/typeweaver
EXPOSE 3000

ENTRYPOINT ["typeweaver", "serve", "--registry-root", "/data/typeweaver", "--host", "0.0.0.0", "--port", "3000"]
