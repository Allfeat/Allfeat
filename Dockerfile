########
# BASE #
########

# base is the first stage where all the debian dependencies
# needed to build the Allfeat binary are installed,
# plus cargo-check to optimize rust dependency management and then speedup any re-build

FROM rust:bookworm as base

WORKDIR /app

# This installs all debian dependencies we need (besides Rust).
RUN apt update -y && \
    apt install -y build-essential git clang curl libssl-dev \
    llvm libudev-dev make protobuf-compiler pkg-config

# Using cargo-chef to only pay the deps installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef

###########
# PLANNER #
###########

# planner is where chef prepares its recipe using a local cache for the target

FROM base AS planner
COPY . .

RUN --mount=type=cache,mode=0755,target=/app/target cargo chef prepare --recipe-path recipe.json

##########
# CACHER #
##########

# cacher is where chef cooks all the deps from the recipe inside the local cache

FROM base as cacher
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
COPY --from=planner /app/rust-toolchain.toml rust-toolchain.toml
RUN --mount=type=cache,mode=0755,target=/app/target cargo chef cook --release --recipe-path recipe.json

###########
# BUILDER #
###########

# builder is where chef builds the binary using the deps of the cache

FROM cacher AS builder
COPY . .

# Build the binary
# We prioritize the local toolchain file
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release

# Strip the binary to reduce the final image size significantly
# (Removes debug symbols, not needed for production runtime)
RUN --mount=type=cache,target=/app/target \
    strip /app/target/release/allfeat && \
    cp /app/target/release/allfeat /usr/local/bin/allfeat

###########
# RUNTIME #
###########

# runtime is where the Allfeat binary is finally copied from the builder
# inside an autonomous slim and secured image.

FROM debian:bookworm-slim AS runtime

WORKDIR /app

LABEL io.allfeat.image.type="builder" \
    io.allfeat.image.authors="hello@allfeat.com" \
    io.allfeat.image.vendor="Allfeat" \
    io.allfeat.image.description="Multistage Container image of the Allfeat Node." \
    io.allfeat.image.source="https://github.com/allfeat/allfeat/blob/${VCS_REF}/Dockerfile" \
    io.allfeat.image.documentation="https://github.com/allfeat/allfeat"

# Install minimal runtime dependencies including shell
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/allfeat /usr/local/bin


RUN useradd -m -u 1000 -U -s /bin/sh -d /app allfeat && \
    mkdir -p /data /app/.local/share && \
    chown -R allfeat:allfeat /data && \
    ln -s /data /app/.local/share/allfeat && \
    # check if executable works in this container
    /usr/local/bin/allfeat --version

USER allfeat

EXPOSE 30333 9933 9944 9615

VOLUME ["/data"]


ENTRYPOINT ["/usr/local/bin/allfeat"]
