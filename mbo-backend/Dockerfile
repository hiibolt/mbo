# Multi-stage build for MBO Backend using Nix
# Stage 1: Build with Nix
FROM nixos/nix:latest AS builder

# Enable flakes and other experimental features
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

# Set working directory
WORKDIR /build

# Copy flake files first for better caching
COPY flake.nix flake.lock ./

# Copy the backend source
COPY mbo-backend ./mbo-backend

# Build the backend using Nix
RUN nix build .#backend --print-build-logs

# Create output directory and copy binary
RUN mkdir -p /output && \
    cp -L result/bin/mbo /output/mbo

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3 \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 app

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /output/mbo /app/mbo

# Copy assets (DBN file)
COPY mbo-backend/assets /app/assets

# Create data directory for SQLite
RUN mkdir -p /app/data && chown -R app:app /app

# Switch to app user
USER app

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/mbo", "--version"] || exit 1

# Set default environment variables
ENV BIND_ADDRESS=0.0.0.0:3000 \
    DB_PATH=/app/data/mbo_data.db \
    DBN_FILE_PATH=/app/assets/CLX5_mbo.dbn \
    RUST_LOG=info

# Run the application
CMD ["/app/mbo"]
