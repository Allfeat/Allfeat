# Multi-stage Dockerfile for Allfeat Substrate Node
# Optimized for production builds and CI/CD pipelines

# base is the first stage where all the debian dependencies
# needed to build the Allfeat binary are installed,
# plus cargo-check to optimize rust dependency management and then speedup any re-build

FROM rust:1.85-slim AS base

# Set working directory
WORKDIR /allfeat

# Copy toolchain configuration
COPY rust-toolchain.toml .
COPY rustfmt.toml .

# Using cargo-chef to only pay the deps installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef

# Install cargo-chef for dependency caching
RUN cargo install cargo-chef --locked

# ================================
# Planner stage
# ================================
FROM base as planner

# Copy manifests
COPY . .

# Generate recipe.json with dependencies
RUN cargo chef prepare --recipe-path recipe.json

# ================================
# Cacher stage
# ================================
FROM base as cacher

# Copy the recipe.json from planner
COPY --from=planner /allfeat/recipe.json recipe.json

# Build dependencies - this layer will be cached
RUN cargo chef cook --release --recipe-path recipe.json

# ================================
# Builder stage
# ================================
FROM cacher as builder

# Copy source code
COPY . .

# Build the node binary (dependencies are already cached)
RUN cargo build --release --locked

# runtime is where the Allfeat binary is finally copied from the builder
# inside an autonomous slim and secured image.

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create allfeat user
RUN groupadd -r allfeat && useradd -r -g allfeat allfeat

# Create data directory
RUN mkdir -p /data && chown allfeat:allfeat /data

# Install minimal runtime dependencies including shell
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/allfeat /usr/local/bin

# Copy and set permissions for entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

RUN useradd -m -u 1000 -U -s /bin/sh -d /app allfeat && \
    mkdir -p /data /app/.local/share && \
    chown -R allfeat:allfeat /data && \
    ln -s /data /app/.local/share/allfeat && \
    # check if executable works in this container
    /usr/local/bin/allfeat --version

# Set proper ownership and permissions
RUN chown allfeat:allfeat /usr/local/bin/allfeat && \
    chmod +x /usr/local/bin/allfeat

# Switch to allfeat user
USER allfeat

# Environment variable to control dev mode
ENV DEV_MODE=false

EXPOSE 30333 9933 9944 9615

# Set data directory as volume
VOLUME ["/data"]


ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
