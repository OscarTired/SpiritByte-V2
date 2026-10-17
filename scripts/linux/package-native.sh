#!/usr/bin/env bash
set -euo pipefail
project_dir=$(cd -- "$(dirname -- "$0")/../.." && pwd)
binary=${1:?Usage: package-native.sh /path/to/spiritbyte /output/directory}
output=${2:?Output directory required}
bundle="$output/SpiritByte_0.1.2_linux-native_x86_64"
mkdir -p "$bundle/licenses"
install -m 755 "$binary" "$bundle/spiritbyte"
install -m 644 "$project_dir/src-tauri/icons/128x128.png" "$bundle/spiritbyte.png"
install -m 755 "$project_dir/scripts/linux/install-native.sh" "$bundle/install-native.sh"
cp "$project_dir/LICENSE" "$bundle/licenses/SpiritByte-LICENSE"
cp "$project_dir/linux-native/assets/"*-LICENSE "$bundle/licenses/"
tar -C "$output" -czf "$bundle.tar.gz" "$(basename "$bundle")"
sha256sum "$bundle.tar.gz"
