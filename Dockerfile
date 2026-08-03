FROM debian:trixie-slim AS base-env

ARG TARGETARCH
ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    FNM_DIR=/usr/local/fnm \
    CARGO_HOME=/usr/local/cargo \
    RUSTUP_HOME=/usr/local/rustup \
    PATH=/usr/local/fnm:/usr/local/cargo/bin:/usr/local/bin:$PATH

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    git \
    build-essential \
    clang \
    clang-tools \
    llvm \
    lld \
    libssl-dev \
    pkg-config \
    flatpak-builder \
    fakeroot \
    zstd \
    zip \
    unzip \
    protobuf-compiler \
    ca-certificates \
### [ DONT THING THIS IS REQUIRED BUT MAYBE IN THE FUTURE] \
##  libgl-dev \
##  libgl1 \
##  qt6-base-dev \
##  qt6-declarative-dev \
##  qt6-tools-dev \
##  qt6-tools-dev-tools \
    fuse \
    && rm -rf /var/lib/apt/lists/*

RUN if [ "$TARGETARCH" = "amd64" ]; then \
        dpkg --add-architecture arm64 && \
        apt-get update && \
        apt-get install -y --no-install-recommends \
            gcc-aarch64-linux-gnu \
            g++-aarch64-linux-gnu; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        dpkg --add-architecture amd64 && \
        apt-get update && \
        apt-get install -y --no-install-recommends \
            gcc-x86-64-linux-gnu \
            g++-x86-64-linux-gnu; \
    fi && \
    rm -rf /var/lib/apt/lists/*

RUN  install -d -m 0755 /etc/apt/keyrings && \
  curl -fsSL https://deb.griffo.io/EA0F721D231FDD3A0A17B9AC7808B4DD62C41256.asc | gpg --dearmor --yes -o /etc/apt/keyrings/deb.griffo.io.gpg && \
  echo "deb [signed-by=/etc/apt/keyrings/deb.griffo.io.gpg] https://deb.griffo.io/apt trixie main" | tee /etc/apt/sources.list.d/deb.griffo.io.list > /dev/null && \
  apt-get update -y && \
  apt-get install zig -y

RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/local/bin \
    && curl -fsSL https://bun.sh/install | bun_install_owner=root BINDIR=/usr/local/bin bash

RUN curl -fsSL https://fnm.vercel.app/install | bash -s -- --install-dir /usr/local/fnm --skip-shell \
    && ln -s /usr/local/fnm/fnm /usr/local/bin/fnm \
    && fnm install --lts \
    && fnm default $(fnm current) \
    && cp -r /usr/local/fnm/aliases/default/bin/* /usr/local/bin/

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain stable \
    && rustup target add \
    x86_64-unknown-linux-gnu \
    x86_64-pc-windows-msvc \
    aarch64-unknown-linux-gnu \
    aarch64-pc-windows-msvc \
    aarch64-apple-darwin \
    && rustup component add llvm-tools

RUN cargo install cargo-zigbuild cargo-xwin mdbook

ENV SQLX_OFFLINE=true
WORKDIR /workspace
CMD ["just", "--list"]
