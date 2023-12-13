FROM rustlang/rust:nightly-slim as builder

ADD . ./workdir
WORKDIR "/workdir"

# This installs all dependencies that we need (besides Rust).
RUN apt update -y && \
    apt install build-essential git clang curl libssl-dev llvm libudev-dev make protobuf-compiler pkg-config -y

# This installs Rust and updates Rust to the right version.
RUN rustup target add wasm32-unknown-unknown --toolchain nightly && rustup show
# This builds the binary.
RUN cargo build --locked --release

# Second stage for a smaller image.
FROM rustlang/rust:nightly-slim

LABEL io.allfeat.image.type="binary" \
    io.allfeat.image.authors="tech@allfeat.com" \
    io.allfeat.image.vendor="Allfeat labs" \
    io.allfeat.image.description="Docker image including allfeat's blockchain binary (testnet-v1)" \
    io.allfeat.image.source="https://github.com/allfeat/allfeat/blob/${VCS_REF}/node.dockerfile" \
    io.allfeat.image.documentation="https://github.com/allfeat/allfeat"

# This installs all dependencies that we need (besides Rust).
RUN apt update -y && apt install build-essential -y

# This installs Rust and updates Rust to the right version.
RUN rustup install stable && rustup component add rust-src --toolchain stable-x86_64-unknown-linux-gnu && rustup show

# This copies the binary from the previous stage.
COPY --from=builder /workdir/target/release/allfeat /usr/local/bin

# Path to the node executable.
ENV CONTRACTS_NODE=/usr/local/bin/allfeat

# Expose needed ports
EXPOSE 30333 9933 9944 9615
