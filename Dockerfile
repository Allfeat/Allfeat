########
# BASE #
########

# base is the first stage where all the debian dependencies 
# needed to build the Allfeat binary are installed, 
# plus cargo-check to optimize rust dependency management and then speedup any re-build

FROM rustlang/rust:nightly-bookworm-slim as base

WORKDIR /app

# This installs all debian dependencies we need (besides Rust).
RUN apt update -y && \
    apt install -y build-essential git clang curl libssl-dev \
    llvm libudev-dev make protobuf-compiler pkg-config

# Using cargo-chef to only pay the deps installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef 

# FIXME: chef raises an error if those 2 deps are not pre-installed
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add rust-src

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
RUN --mount=type=cache,mode=0755,target=/app/target cargo chef cook --release --recipe-path recipe.json

###########
# BUILDER #
###########

# builder is where chef builds the binary using the deps of the cache

FROM cacher AS builder
COPY . .
RUN --mount=type=cache,mode=0755,target=/app/target cargo build --locked --release
RUN --mount=type=cache,mode=0755,target=/app/target cp /app/target/release/allfeat /usr/local/bin

###########
# RUNTIME #
###########

# runtime is where the Allfeat binary is finally copied from the builder 
# inside an autonomous slim and secured image.

FROM debian:bookworm-slim AS runtime

WORKDIR /app

LABEL io.allfeat.image.type="builder" \
    io.allfeat.image.authors="tech@allfeat.com" \
    io.allfeat.image.vendor="Allfeat labs" \
    io.allfeat.image.description="Multistage Docker image for allfeat-blockchain" \
    io.allfeat.image.source="https://github.com/allfeat/allfeat/blob/${VCS_REF}/Dockerfile" \
    io.allfeat.image.documentation="https://github.com/allfeat/allfeat"

COPY --from=builder /usr/local/bin/allfeat /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /app allfeat && \
    mkdir -p /data /app/.local/share && \
    chown -R allfeat:allfeat /data && \
    ln -s /data /app/.local/share/allfeat && \
    # unclutter and minimize the attack surface
    rm -rf /usr/bin /usr/sbin && \
    # check if executable works in this container
    /usr/local/bin/allfeat --version

USER allfeat

EXPOSE 30333 9933 9944 9615

VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/allfeat", "--dev"]
