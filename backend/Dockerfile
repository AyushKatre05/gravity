# ─── Stage 1: Builder ────────────────────────────────────────────────────────
FROM rust:1.85-slim AS builder

# Install essential build tooling
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies first for faster rebuilds
COPY Cargo.toml Cargo.lock* ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN SQLX_OFFLINE=true cargo build --release 2>/dev/null || true
RUN rm -f src/main.rs

# Copy real source
COPY . .

# Build with sqlx offline mode (no live DB needed at build time)
ENV SQLX_OFFLINE=true
RUN cargo build --release

# ─── Stage 2: Runtime ────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/gravity-backend /usr/local/bin/gravity-backend

# Copy migrations so sqlx::migrate!() finds them at runtime
COPY --from=builder /app/migrations ./migrations

# Create the /analyze mount point
RUN mkdir -p /analyze

# Render and other platforms inject PORT; default to 8080
ENV PORT=8080
ENV ANALYZE_PATH=/analyze

EXPOSE ${PORT}

HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:${PORT}/health || exit 1

CMD ["gravity-backend"]
