FROM node:22-bookworm-slim AS node

# Build on an older glibc baseline for portability to newer Linux distributions.
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl build-essential pkg-config file patchelf \
    libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev \
    librsvg2-dev libxdo-dev libfuse2 xdg-utils \
    && rm -rf /var/lib/apt/lists/*
COPY --from=node /usr/local/ /usr/local/
RUN npm install --global pnpm@9.15.9 \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup.sh \
    && sh /tmp/rustup.sh -y --profile minimal --default-toolchain stable \
    && rm /tmp/rustup.sh
ENV PATH="/root/.cargo/bin:${PATH}" \
    CARGO_TARGET_DIR=/linux-target \
    APPIMAGE_EXTRACT_AND_RUN=1 \
    CI=true
COPY build.sh /usr/local/bin/build-spiritbyte
RUN sed -i 's/\r$//' /usr/local/bin/build-spiritbyte \
    && chmod +x /usr/local/bin/build-spiritbyte
ENTRYPOINT ["/usr/local/bin/build-spiritbyte"]
