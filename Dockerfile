# Build stage - use latest Rust with Debian bookworm
FROM rust:bookworm as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Copy source code
COPY src ./src
COPY templates ./templates
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    yt-dlp \
    gosu \
    python3 \
    python3-pip \
    python3-venv \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# Install spotdl via pip
RUN pip3 install --no-cache-dir --break-system-packages spotdl

# Create app user
RUN useradd -m -u 1000 app

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/j3s_music_upload /app/j3s_music_upload

# Copy templates and migrations
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/migrations /app/migrations

# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Create necessary directories
RUN mkdir -p /app/data /app/templates /app/migrations /srv/navidrome/music /srv/navidrome/music/tmp

# Change ownership of app directory
RUN chown -R app:app /app /srv/navidrome

# Note: We don't switch to app user here - entrypoint.sh will do it
# This allows the entrypoint to fix permissions on mounted volumes

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV CONFIG_PATH=/app/config.toml

# Use entrypoint script to handle permissions
ENTRYPOINT ["/entrypoint.sh"]

# Run the application
CMD ["/app/j3s_music_upload"]
