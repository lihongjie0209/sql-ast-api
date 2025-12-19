# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code and static files
COPY src ./src
COPY static ./static

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install required libraries
RUN apt-get update && \
    apt-get install -y \
    libssl3 \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/sql-ast-api /usr/local/bin/sql-ast-api

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the binary
ENTRYPOINT ["sql-ast-api"]
CMD ["--host", "0.0.0.0"]
