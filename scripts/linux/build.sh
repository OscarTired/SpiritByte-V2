#!/usr/bin/env bash
set -euo pipefail

# Copy only sources into the Linux filesystem; Windows dependencies and Rust
# artifacts cannot be reused here. The source mounts remain read-only.
mkdir -p /build/SpiritByte-V2
tar -C /source/desktop --exclude=.git --exclude=node_modules \
    --exclude=dist --exclude=target -cf - . \
    | tar -C /build/SpiritByte-V2 -xf -

cd /build/SpiritByte-V2
pnpm install --frozen-lockfile
pnpm tauri build --bundles appimage,deb -- --locked

mkdir -p /output/appimage /output/deb
cp /linux-target/release/bundle/appimage/*.AppImage /output/appimage/
cp /linux-target/release/bundle/deb/*.deb /output/deb/
sha256sum /output/appimage/*.AppImage /output/deb/*.deb
