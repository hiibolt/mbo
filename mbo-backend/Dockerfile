# Multi-stage build using Rust 1.91 (supports edition2024)
FROM rust:1.84-bookworm AS builder

# Install Rust 1.91
RUN rustup install 1.91 && rustup default 1.91

WORKDIR /build

# Copy everything needed for the build
COPY mbo-backend ./

# Build the application in release mode
RUN cargo build --release

# Runtime stage with minimal Debian base
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    sqlite3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash app

WORKDIR /app

# Copy the built binary from the builder
COPY --from=builder /build/target/release/mbo /app/mbo

# Copy assets
COPY mbo-backend/assets /app/assets

# Create data directory for SQLite
RUN mkdir -p /app/data && chown app:app /app/data

USER app

# Set default environment variables
ENV BIND_ADDRESS=0.0.0.0:3000
ENV DB_PATH=/app/data/mbo.db
ENV DBN_FILE_PATH=/app/assets/CLX5_mbo.dbn
ENV RUST_LOG=info

EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 CMD ["/app/mbo", "--version"]

CMD ["/app/mbo"]