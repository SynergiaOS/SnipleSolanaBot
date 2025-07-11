FROM rust:1.75-slim as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN cargo build --release

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/snipercor /usr/local/bin/snipercor
RUN chmod +x /usr/local/bin/snipercor
EXPOSE 8080
CMD ["/usr/local/bin/snipercor"]
