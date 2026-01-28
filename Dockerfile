# Build Stage
FROM rust:1.84 as builder
WORKDIR /app
COPY . .
# Build for release
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app

# Install SSL certificates (required for talking to Google API)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/gmail-api-proxy /usr/local/bin/gmail-api-proxy

# Set default environment
ENV RUST_LOG=info
ENV PORT=3000

expose 3000

CMD ["gmail-api-proxy"]
