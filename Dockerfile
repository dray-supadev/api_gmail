# Stage 1: Build Frontend
FROM node:20-slim as frontend-builder
WORKDIR /app

# Copy package files first to leverage cache
COPY frontend/package.json frontend/package-lock.json ./
RUN npm install

# Copy source and build
COPY frontend/ ./
# Set correct path for vite build
RUN npm run build
# Explicitly copy embed.js to ensure it ends up in dist
RUN cp public/embed.js dist/embed.js || echo "Warning: public/embed.js empty or missing"

# Stage 2: Build Backend
FROM rust:1.84 as backend-builder
WORKDIR /app
COPY . .
# Build for release
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
WORKDIR /app

# Install SSL certificates (required for talking to Google API)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=backend-builder /app/target/release/gmail-api-proxy /usr/local/bin/gmail-api-proxy

# Copy the frontend build artifacts
# Make sure the directory structure matches what main.rs expects ("frontend/dist")
COPY --from=frontend-builder /app/dist /app/frontend/dist

# Set default environment
ENV RUST_LOG=info
ENV PORT=3000

EXPOSE 3000

CMD ["gmail-api-proxy"]
