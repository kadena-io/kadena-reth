ARG UBUNTU_VERSION=22.04
ARG PROJECT_NAME=kadena-reth

# ############################################################################ #
# Base Environment

FROM ubuntu:${UBUNTU_VERSION} AS reth-base
ARG TARGETPLATFORM
ARG BUILDPLATFORM
RUN <<EOF
  echo "BUILDPLATFORM: $BUILDPLATFORM"
  echo "TARGETPLATFORM: $TARGETPLATFORM"
EOF
WORKDIR /app
LABEL org.opencontainers.image.source=https://github.com/kadena-io/eth-pow-beacon
LABEL org.opencontainers.image.licenses="MIT"

# ############################################################################ #
# Build Environment

FROM reth-base AS reth-rust
ARG TARGETPLATFORM

# System Packages
ENV DEBIAN_FRONTEND=noninteractive
RUN <<EOF
    apt-get update -y
    apt-get install -yqq \
        binutils \
        build-essential\
        curl \
        libclang-dev \
        libssl-dev \
        pkg-config
EOF

# RUN echo "pong" > /tmp/pong

# Rust Toolchain
ENV PATH="${PATH}:/root/.cargo/bin"
RUN --mount=type=cache,target=/root/.cargo,id=cargo-${TARGETPLATFORM} <<EOF
  curl -sSf https://sh.rustup.rs | sh -s -- -y
  # curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C /root/.cargo/bin
  cargo version
  rustc --version
EOF

# ############################################################################ #
# Build Context

FROM reth-rust AS reth-build-ctx
COPY . .
ENV GIT_DISCOVERY_ACROSS_FILESYSTEM=1
RUN mkdir -p /tools
COPY --chmod=0755 <<EOF /tools/check-git-clean.sh
#!/bin/sh
if [ -d ".git" ] && ! [ -f "/tools/wip" ] && ! git diff --exit-code; then \
    echo "Git working tree is not clean. The build changed some file that is checked into git." 1>&2 ; \
    exit 1 ; \
fi
EOF
RUN sh /tools/check-git-clean.sh || touch /tools/wip

# ############################################################################ #
# Build Kadena Reth

FROM reth-build-ctx AS reth-build
ARG TARGETPLATFORM
ARG PROJECT_NAME
ARG BUILD_PROFILE=release
ARG RUSTFLAGS=""
ARG FEATURES=""
ENV BUILD_PROFILE=$BUILD_PROFILE
ENV RUSTFLAGS="$RUSTFLAGS"
ENV FEATURES=$FEATURES

RUN --mount=type=cache,target=/root/.cargo,id=cargo-${TARGETPLATFORM} \
    --mount=type=cache,target=/app/target,id=cargo-${PROJECT_NAME}-${TARGETPLATFORM},sharing=locked <<EOF
    cargo build --profile="$BUILD_PROFILE" --features "$FEATURES"  --bin kadena-reth
EOF
RUN --mount=type=cache,target=/root/.cargo,id=cargo-${TARGETPLATFORM} \
    --mount=type=cache,target=/app/target,id=cargo-${PROJECT_NAME}-${TARGETPLATFORM},sharing=locked <<EOF
    cargo build --profile="$BUILD_PROFILE" --features "$FEATURES"  --bin kadena-reth
EOF
RUN --mount=type=cache,target=/root/.cargo,id=cargo-${TARGETPLATFORM} \
    --mount=type=cache,target=/app/target,id=cargo-${PROJECT_NAME}-${TARGETPLATFORM},sharing=locked <<EOF
    cp /app/target/$BUILD_PROFILE/kadena-reth /app/kadena-reth
EOF
RUN sh /tools/check-git-clean.sh

# ############################################################################ #
# Runtime Image

FROM reth-base AS reth-dist
ENV PATH="/app:${PATH}"
COPY --from=reth-build /app/kadena-reth /app/kadena-reth
EXPOSE 30303 30303/udp 9001 8545 8546
ENTRYPOINT ["/app/kadena-reth"]
LABEL org.opencontainers.image.source=https://github.com/kadena-io/eth-pow-beacon
LABEL org.opencontainers.image.licenses="MIT"

