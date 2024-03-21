# This is the first stage. Here we install all the dependencies that we need in order to build the Allfeat binary
# and we create the Allfeat binary in a rust oriented temporary image.
FROM rust:slim-buster as builder

ADD . ./workdir
WORKDIR "/workdir"

# This installs all dependencies that we need (besides Rust).
RUN apt update -y && \
    apt install build-essential git clang curl libssl-dev llvm libudev-dev make protobuf-compiler pkg-config -y

# This builds the binary.
RUN cargo build --locked --release

# This is the 2nd stage: a very small image where we copy the Allfeat binary."
FROM docker.io/library/ubuntu:20.04

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