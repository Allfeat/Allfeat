########
# BASE #
########

FROM rust:bookworm AS base

WORKDIR /app

RUN apt-get update -y && \
    apt-get install -y --no-install-recommends \
      build-essential git clang curl libssl-dev \
      llvm libudev-dev make protobuf-compiler pkg-config libclang-dev \
      mold xz-utils && \
    rm -rf /var/lib/apt/lists/*

RUN ARCH=$(uname -m) && \
    case "$ARCH" in \
      x86_64)  TARGET="x86_64-unknown-linux-musl" ;; \
      aarch64) TARGET="aarch64-unknown-linux-musl" ;; \
      *) echo "unsupported arch: $ARCH" && exit 1 ;; \
    esac && \
    curl -sSLf "https://github.com/LukeMathWalker/cargo-chef/releases/download/v0.1.77/cargo-chef-${TARGET}.tar.xz" \
    | tar -xJ --strip-components=1 -C /usr/local/bin "cargo-chef-${TARGET}/cargo-chef"

# cacher and builder must share the same linker or cargo treats the artifacts as stale
ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"

###########
# PLANNER #
###########

FROM base AS planner
COPY . .

RUN cargo chef prepare --recipe-path recipe.json

##########
# CACHER #
##########

FROM base AS cacher
COPY --from=planner /app/recipe.json recipe.json
COPY --from=planner /app/rust-toolchain.toml rust-toolchain.toml

RUN cargo chef cook --release --locked --recipe-path recipe.json

###########
# BUILDER #
###########

FROM cacher AS builder
COPY . .

RUN cargo build --locked --release

RUN strip /app/target/release/allfeat && \
    cp /app/target/release/allfeat /usr/local/bin/allfeat

###########
# RUNTIME #
###########

FROM debian:bookworm-slim AS runtime

WORKDIR /app

LABEL io.allfeat.image.type="builder" \
    io.allfeat.image.authors="hello@allfeat.com" \
    io.allfeat.image.vendor="Allfeat" \
    io.allfeat.image.description="Multistage Container image of the Allfeat Node." \
    io.allfeat.image.source="https://github.com/allfeat/allfeat/blob/${VCS_REF}/Dockerfile" \
    io.allfeat.image.documentation="https://github.com/allfeat/allfeat"

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/allfeat /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /app allfeat && \
    mkdir -p /data /app/.local/share && \
    chown -R allfeat:allfeat /data && \
    ln -s /data /app/.local/share/allfeat && \
    /usr/local/bin/allfeat --version

USER allfeat

EXPOSE 30333 9933 9944 9615

VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/allfeat"]
