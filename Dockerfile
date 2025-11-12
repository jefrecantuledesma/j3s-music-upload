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
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 app

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/j3s_music_upload /app/j3s_music_upload

# Copy templates and migrations
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/migrations /app/migrations

# Create necessary directories
RUN mkdir -p /app/data /app/templates /app/migrations /srv/navidrome/music /srv/navidrome/music/tmp

# Change ownership
RUN chown -R app:app /app /srv/navidrome

# Switch to app user
USER app

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV CONFIG_PATH=/app/config.toml

# Run the application
CMD ["/app/j3s_music_upload"]
