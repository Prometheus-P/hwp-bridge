# ============================================
# HwpBridge Dockerfile
# Multi-stage build for minimal image size
# ============================================

# ============================================
# Stage 1: Build
# ============================================
FROM rust:1.85-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/hwp-types/Cargo.toml crates/hwp-types/
COPY crates/hwp-core/Cargo.toml crates/hwp-core/
COPY crates/hwp-cli/Cargo.toml crates/hwp-cli/
COPY crates/hwp-web/Cargo.toml crates/hwp-web/
COPY crates/hwp-mcp/Cargo.toml crates/hwp-mcp/

# Create dummy source files for dependency caching
RUN mkdir -p crates/hwp-types/src && echo "pub fn dummy() {}" > crates/hwp-types/src/lib.rs
RUN mkdir -p crates/hwp-core/src && echo "pub fn dummy() {}" > crates/hwp-core/src/lib.rs
RUN mkdir -p crates/hwp-cli/src && echo "fn main() {}" > crates/hwp-cli/src/main.rs
RUN mkdir -p crates/hwp-web/src && echo "fn main() {}" > crates/hwp-web/src/main.rs
RUN mkdir -p crates/hwp-mcp/src && echo "fn main() {}" > crates/hwp-mcp/src/main.rs

# Build dependencies only
RUN cargo build --release --workspace

# Remove dummy files
RUN rm -rf crates/*/src

# Copy actual source code
COPY crates crates

# Touch files to invalidate cache for actual build
RUN touch crates/hwp-types/src/lib.rs
RUN touch crates/hwp-core/src/lib.rs
RUN touch crates/hwp-cli/src/main.rs
RUN touch crates/hwp-web/src/main.rs
RUN touch crates/hwp-mcp/src/main.rs

# Build release
RUN cargo build --release --workspace

# ============================================
# Stage 2: hwp-web runtime
# ============================================
FROM debian:bookworm-slim AS hwp-web

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false hwp

# Copy binary
COPY --from=builder /app/target/release/hwp-web /usr/local/bin/hwp-web

# Set ownership
RUN chown hwp:hwp /usr/local/bin/hwp-web

# Switch to non-root user
USER hwp

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Environment
ENV RUST_LOG=info
ENV PORT=3000

# Run
CMD ["hwp-web"]

# ============================================
# Stage 3: hwp-mcp runtime
# ============================================
FROM debian:bookworm-slim AS hwp-mcp

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false hwp

# Copy binary
COPY --from=builder /app/target/release/hwp-mcp /usr/local/bin/hwp-mcp

# Set ownership
RUN chown hwp:hwp /usr/local/bin/hwp-mcp

# Switch to non-root user
USER hwp

# Environment
ENV RUST_LOG=info

# Run (stdio mode)
CMD ["hwp-mcp"]

# ============================================
# Stage 4: hwp-cli runtime
# ============================================
FROM debian:bookworm-slim AS hwp-cli

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/hwp-cli /usr/local/bin/hwp-cli

# Default entrypoint
ENTRYPOINT ["hwp-cli"]
CMD ["--help"]

# ============================================
# Stage 5: All-in-one (for development)
# ============================================
FROM debian:bookworm-slim AS all

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false hwp

# Copy all binaries
COPY --from=builder /app/target/release/hwp-cli /usr/local/bin/
COPY --from=builder /app/target/release/hwp-web /usr/local/bin/
COPY --from=builder /app/target/release/hwp-mcp /usr/local/bin/

# Set ownership
RUN chown -R hwp:hwp /usr/local/bin/hwp-*

# Switch to non-root user
USER hwp

# Default to web server
EXPOSE 3000
ENV RUST_LOG=info

CMD ["hwp-web"]
