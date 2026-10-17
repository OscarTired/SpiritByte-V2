#!/usr/bin/env bash
set -euo pipefail
# Linux is a native GTK client; Windows continues to build src-tauri.
mkdir -p /build/SpiritByte-V2
tar -C /source/desktop --exclude=.git --exclude=node_modules \
    --exclude=dist --exclude=target -cf - . \
    | tar -C /build/SpiritByte-V2 -xf -
cd /build/SpiritByte-V2
cargo test --manifest-path linux-native/Cargo.toml --locked
cargo test --manifest-path core/Cargo.toml
cargo build --manifest-path linux-native/Cargo.toml --release --locked
bash scripts/linux/package-native.sh /linux-target/release/spiritbyte /output
