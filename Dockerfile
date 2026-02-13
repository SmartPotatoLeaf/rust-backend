# Stage 1: Chef - Compute the recipe
# Using Rust nightly for edition2024 support (required by time crate)
FROM rustlang/rust:nightly-slim AS chef
WORKDIR /app

# Install build dependencies in a single layer
RUN apt-get update && apt-get install -y \
    lld \
    clang \
    pkg-config \
    libssl-dev \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-chef for dependency caching
RUN cargo install cargo-chef --locked

# Stage 2: Planner - Generate the recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - Build dependencies and application
FROM chef AS builder

# Copy the recipe and build dependencies (cached layer)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY . .

# Build optimized binary
# Note: All optimization flags are in Cargo.toml [profile.release]
# This avoids conflicts between RUSTFLAGS and profile settings
# For smallest binary, use: cargo build --profile release-small
RUN cargo build --release --bin spl-server

# Stage 4: Runtime - Distroless image for maximum security and minimal size
FROM gcr.io/distroless/cc-debian12:nonroot

# Copy the binary from builder
COPY --from=builder --chown=nonroot:nonroot /app/target/release/spl-server /usr/local/bin/spl-server

# Copy configuration files (if needed)
COPY --from=builder --chown=nonroot:nonroot /app/config /app/config

# Set working directory
WORKDIR /app

# Use non-root user (already default in distroless nonroot variant)
USER nonroot:nonroot

# Health check (optional - comment out if not needed in container orchestrator)
# HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
#   CMD ["/usr/local/bin/spl-server", "--health-check"]

# Expose default port (documentation only, actual port from config)
EXPOSE 8080

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/spl-server"]
