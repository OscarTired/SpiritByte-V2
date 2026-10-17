# Older glibc baseline; the installed native client uses the host's GTK 3.
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl build-essential pkg-config libgtk-3-dev \
    libfontconfig1-dev librsvg2-common \
    && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup.sh \
    && sh /tmp/rustup.sh -y --profile minimal --default-toolchain stable \
    && rm /tmp/rustup.sh
ENV PATH="/root/.cargo/bin:${PATH}" \
    CARGO_TARGET_DIR=/linux-target \
    CARGO_BUILD_JOBS=1 \
    CI=true
COPY build.sh /usr/local/bin/build-spiritbyte
RUN sed -i 's/\r$//' /usr/local/bin/build-spiritbyte \
    && chmod +x /usr/local/bin/build-spiritbyte
ENTRYPOINT ["/usr/local/bin/build-spiritbyte"]
