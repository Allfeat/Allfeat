########
# BASE #
########

# base is the first stage where we install all the debian dependencies 
# that we need to build the Allfeat binary, plus cargo-check to
# optimize rust dependency management and then speedup re-build

FROM rustlang/rust:nightly-bookworm-slim as builder

# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef 

# This installs all dependencies that we need (besides Rust).
RUN apt update -y && \
    apt install -y build-essential git clang curl libssl-dev \
    llvm libudev-dev make protobuf-compiler pkg-config

WORKDIR workdir

###########
# PLANNER #
###########

# planner is where chef prepare its recipe

FROM base AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

###########
# BUILDER #
###########

# builder is where chef build all the deps
# and then create the Allfeat binary in a rust oriented temporary image.

FROM base AS builder

# prepare recipe and other deps
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add rust-src
COPY --from=planner /workdir/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --locked --release

###########
# RUNTIME #
###########

# This is the 2nd stage: a very small image where we copy the Allfeat binary."

FROM debian:bookworm-slim AS runtime

WORKDIR workdir

LABEL io.allfeat.image.type="builder" \
    io.allfeat.image.authors="tech@allfeat.com" \
    io.allfeat.image.vendor="Allfeat labs" \
    io.allfeat.image.description="Multistage Docker image for allfeat-blockchain" \
    io.allfeat.image.source="https://github.com/allfeat/allfeat/blob/${VCS_REF}/Dockerfile" \
    io.allfeat.image.documentation="https://github.com/allfeat/allfeat"

COPY --from=builder /workdir/target/release/allfeat /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /workdir allfeat && \
    mkdir -p /data /workdir/.local/share && \
    chown -R allfeat:allfeat /data && \
    ln -s /data /workdir/.local/share/allfeat && \
    # unclutter and minimize the attack surface
    rm -rf /usr/bin /usr/sbin && \
    # check if executable works in this container
    /usr/local/bin/allfeat --version

USER allfeat

EXPOSE 30333 9933 9944 9615

VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/allfeat", "--dev"]
