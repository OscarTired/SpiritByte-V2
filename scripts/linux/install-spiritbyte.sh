#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "Usage: bash $0 /path/to/SpiritByte.AppImage" >&2
    exit 2
fi
source_app=$(realpath "$1")
test -f "$source_app"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
install_dir="$HOME/Applications"
target_app="$install_dir/SpiritByte.AppImage"
icon_path="$data_home/icons/hicolor/128x128/apps/spiritbyte.png"
desktop_path="$data_home/applications/com.spiritbyte.app.desktop"
for path in "$target_app" "$icon_path" "$desktop_path"; do
    if [[ "$path" = *$'\n'* || "$path" = *$'\r'* ]]; then
        echo 'Installation paths must not contain newlines.' >&2
        exit 2
    fi
done
work_dir=$(mktemp -d /tmp/spiritbyte-install.XXXXXX)
trap 'rm -rf "$work_dir"' EXIT
mkdir -p "$install_dir" "$(dirname "$icon_path")" "$(dirname "$desktop_path")"
if [ "$source_app" != "$target_app" ]; then
    install -m 755 "$source_app" "$target_app"
else
    chmod +x "$target_app"
fi
(cd "$work_dir" && "$target_app" --appimage-extract > /dev/null)
install -m 644 "$work_dir/squashfs-root/usr/share/icons/hicolor/128x128/apps/spiritbyte.png" "$icon_path"

# Escape both the desktop-entry string and its quoted Exec argument.
quote_exec_path() {
    local value="$1" char i
    printf '"'
    for ((i=0; i<${#value}; i++)); do
        char="${value:i:1}"
        if [[ "$char" = '\' ]]; then
            printf '%s' '\\\\'
        elif [[ "$char" = '"' || "$char" = '$' || "$char" = $'\x60' ]]; then
            printf '%s%s' '\\' "$char"
        elif [[ "$char" = '%' ]]; then
            printf '%s' '%%'
        else
            printf '%s' "$char"
        fi
    done
    printf '"'
}
{
    printf '%s\n' '[Desktop Entry]' 'Type=Application' 'Version=1.0' 'Name=SpiritByte'
    printf '%s\n' 'Comment=Local password manager' 'Comment[es]=Gestor local de contraseñas'
    printf 'Exec=/usr/bin/env -u WEBKIT_DISABLE_DMABUF_RENDERER -u WEBKIT_DISABLE_COMPOSITING_MODE '
    quote_exec_path "$target_app"
    printf '\nIcon=spiritbyte\nTerminal=false\nCategories=Utility;Security;\nKeywords=password;vault;contraseñas;\n'
} > "$desktop_path"
chmod 644 "$desktop_path"
if command -v update-desktop-database > /dev/null; then
    update-desktop-database "$data_home/applications"
fi
echo "Installed: $target_app"
echo "Launcher entry: $desktop_path"
echo 'Open your application launcher and search for SpiritByte.'
