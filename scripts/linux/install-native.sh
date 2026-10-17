#!/usr/bin/env bash
set -euo pipefail
if [ "$#" -ne 1 ]; then
    echo "Usage: bash $0 /path/to/native-bundle" >&2
    exit 2
fi
bundle=$(realpath "$1")
test -x "$bundle/spiritbyte"
test -f "$bundle/spiritbyte.png"
dependencies=$(ldd "$bundle/spiritbyte")
if ! grep -q 'libgtk-3' <<< "$dependencies"; then
    echo 'Expected a native GTK 3 executable.' >&2
    exit 1
fi
# Reject accidentally installing the old webview binary as the native client.
if grep -Eqi 'webkit|javascriptcore' <<< "$dependencies"; then
    echo 'Expected the native GTK client, found a WebKit dependency.' >&2
    exit 1
fi
if grep -q 'not found' <<< "$dependencies"; then
    echo 'Missing native runtime libraries. Install GTK 3 and its image loaders first.' >&2
    printf '%s\n' "$dependencies" >&2
    exit 1
fi
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
install_dir="$HOME/Applications/SpiritByte"
target="$install_dir/spiritbyte"
desktop="$data_home/applications/com.spiritbyte.app.desktop"
for path in "$target" "$desktop"; do
    if [[ "$path" = *$'\n'* || "$path" = *$'\r'* ]]; then echo 'Invalid installation path.' >&2; exit 2; fi
done
mkdir -p "$install_dir" "$data_home/applications" "$data_home/icons/hicolor/128x128/apps"
install -m 755 "$bundle/spiritbyte" "$install_dir/spiritbyte.new"
install -m 644 "$bundle/spiritbyte.png" "$data_home/icons/hicolor/128x128/apps/spiritbyte.png"
if [ -d "$bundle/licenses" ]; then cp -a "$bundle/licenses" "$install_dir/"; fi
quote_exec_path() {
    local value="$1" char i
    printf '"'
    for ((i=0; i<${#value}; i++)); do
        char="${value:i:1}"
        if [[ "$char" = '\' ]]; then printf '%s' '\\\\'
        elif [[ "$char" = '"' || "$char" = '$' || "$char" = $'\x60' ]]; then printf '%s%s' '\\' "$char"
        elif [[ "$char" = '%' ]]; then printf '%s' '%%'
        else printf '%s' "$char"; fi
    done
    printf '"'
}
{
    printf '%s\n' '[Desktop Entry]' 'Type=Application' 'Version=1.0' 'Name=SpiritByte'
    printf '%s\n' 'Comment=Native local password manager' 'Comment[es]=Gestor nativo local de contraseñas'
    printf 'Exec='; quote_exec_path "$target"
    printf '\nIcon=spiritbyte\nTerminal=false\nCategories=Utility;Security;\nKeywords=password;vault;contraseñas;\nStartupWMClass=com.spiritbyte.app\n'
} > "$desktop.new"
chmod 644 "$desktop.new"
mv -f "$install_dir/spiritbyte.new" "$target"
mv -f "$desktop.new" "$desktop"
# Retire the installed AppImage only after the native executable and launcher
# have been installed. Keep one rollback copy outside Applications. Vault data
# in com.spiritbyte.app is never read, moved, converted or deleted here.
old="$HOME/Applications/SpiritByte.AppImage"
if [ -f "$old" ]; then
    backup_dir="$data_home/spiritbyte-install-backups"
    mkdir -p "$backup_dir"
    backup="$backup_dir/SpiritByte.AppImage.$(date +%s)"
    mv -- "$old" "$backup"
    echo "Previous installation archived: $backup"
fi
if command -v update-desktop-database >/dev/null; then update-desktop-database "$data_home/applications"; fi
echo "Installed native client: $target"
echo "Launcher: $desktop"
