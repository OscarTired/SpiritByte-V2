#!/usr/bin/env python3
"""Regenerate tiny raw GTK assets from the unchanged Windows artwork.

Development only: ImageMagick is not needed to build or run SpiritByte.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[2]
assets = root / "linux-native/assets"
fox = root / "public/fox-byte.svg"
icon = root / "src-tauri/icons/128x128.png"
with tempfile.TemporaryDirectory(prefix="spiritbyte-art-") as directory:
    styled = Path(directory) / "fox.svg"
    styled.write_text(fox.read_text().replace("viewBox=", 'style="stroke:white;stroke-width:2.2;stroke-linejoin:round;stroke-opacity:.85" viewBox=', 1))
    subprocess.run(["magick", "-background", "none", str(styled), "-resize", "360x360", "-depth", "8", "rgba:" + str(assets / "fox-source.rgba")], check=True)
    subprocess.run(["magick", str(icon), "-depth", "8", "rgba:" + str(assets / "window-icon.rgba")], check=True)
rows = []
for source, output, width in [(fox, "fox-source.rgba", 360), (icon, "window-icon.rgba", 128)]:
    data = (assets / output).read_bytes()
    assert len(data) == width * width * 4
    rows.append({"source": str(source.relative_to(root)), "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                 "output": output, "output_sha256": hashlib.sha256(data).hexdigest(), "width": width, "height": width, "format": "RGBA8"})
(assets / "native-art.json").write_text(json.dumps(rows, indent=2) + "\n")
