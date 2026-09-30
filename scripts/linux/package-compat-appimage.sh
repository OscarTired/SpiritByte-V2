#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    echo "Usage: $0 INPUT.AppImage OUTPUT.AppImage [compat|displayfix]" >&2
    exit 2
fi

mode="${3:-compat}"
case "$mode" in
    compat|displayfix) ;;
    *) echo "Unknown package mode: $mode" >&2; exit 2 ;;
esac

input_path=$(realpath "$1")
output_path=$(realpath -m "$2")
work_dir=$(mktemp -d /tmp/spiritbyte-appimage-compat.XXXXXX)
trap 'rm -rf "$work_dir"' EXIT

cd "$work_dir"
"$input_path" --appimage-extract > /dev/null
app_dir="$work_dir/squashfs-root"

# The host's Mesa must use the host's display-stack libraries. Bundling older
# copies from Ubuntu 22.04 can abort WebKit's render process on rolling distros.
# See https://github.com/tauri-apps/tauri/issues/15976.
for library in \
    'libwayland-client.so*' 'libwayland-cursor.so*' \
    'libwayland-egl.so*' 'libwayland-server.so*' 'libxkbcommon.so*' \
    'libxcb-randr.so*' 'libxcb-render.so*' 'libxcb-shm.so*' \
    'libXau.so*' 'libXdmcp.so*'; do
    find "$app_dir/usr/lib" -maxdepth 1 -name "$library" -print -delete
done

# Both variants use host display libraries. Only compat forces software
# compositing; displayfix leaves WebKit's rendering defaults untouched.
rm -f "$app_dir/apprun-hooks/spiritbyte-compat.sh"
if [ "$mode" = compat ]; then
    mkdir -p "$app_dir/apprun-hooks"
    cat > "$app_dir/apprun-hooks/spiritbyte-compat.sh" <<'HOOK'
#!/usr/bin/env bash
export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
export WEBKIT_DISABLE_COMPOSITING_MODE="${WEBKIT_DISABLE_COMPOSITING_MODE:-1}"
HOOK
    chmod +x "$app_dir/apprun-hooks/spiritbyte-compat.sh"
fi

plugin="$work_dir/linuxdeploy-plugin-appimage.AppImage"
curl -fLsS \
    https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous/linuxdeploy-plugin-appimage-x86_64.AppImage \
    -o "$plugin"
chmod +x "$plugin"
mkdir -p "$(dirname "$output_path")"
APPIMAGE_EXTRACT_AND_RUN=1 LDAI_OUTPUT="$work_dir/SpiritByte-compat.AppImage" \
    "$plugin" --appdir "$app_dir"
mv -f "$work_dir/SpiritByte-compat.AppImage" "$output_path"
