# HwpBridge MCP - Smithery custom container runtime
# Requirements: expose MCP Streamable HTTP on /mcp and listen on $PORT (Smithery sets this).

FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    pkg-config \
    libssl-dev \
  && rm -rf /var/lib/apt/lists/*

# Copy full workspace
COPY . .

# Build only the MCP server binary
RUN cargo build --release -p hwp-mcp

FROM debian:bookworm-slim AS hwp-mcp
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
  && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 10001 hwp
COPY --from=builder /app/target/release/hwp-mcp /usr/local/bin/hwp-mcp
RUN chown hwp:hwp /usr/local/bin/hwp-mcp

USER hwp

# Default envs (Smithery will override PORT)
ENV RUST_LOG=info
ENV HWP_MCP_TRANSPORT=http
ENV PORT=8081

EXPOSE 8081

ENTRYPOINT ["hwp-mcp"]
