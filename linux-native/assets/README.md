# Embedded Linux fonts

The `.woff2` files are the existing SpiritByte font sources. Their `.ttf`
counterparts are lossless SFNT conversions for Fontconfig/Pango: this platform
accepted WOFF2 registration without exposing the families to Pango. The GTK
client embeds and registers the TTF files for its process only. Windows font
assets and rendering are unchanged. Existing font licenses apply to both forms.

To regenerate with Python FontTools and Brotli installed:

```python
from pathlib import Path
from fontTools.ttLib import TTFont

for source in Path("linux-native/assets").glob("*.woff2"):
    font = TTFont(source)
    font.flavor = None
    font.save(source.with_suffix(".ttf"))
```

Neither Python nor FontTools is required to build or run the application.

## Native startup artwork

`fox-source.rgba` is the 360 × 360 RGBA8 raster of the existing Windows
`public/fox-byte.svg`, with the same white facet seams as its splash. Native
Cairo applies the current palette and Bayer dithering once at startup.
`window-icon.rgba` contains the unchanged 128px application icon. Embedding these
small raw resources avoids starting and retaining system PNG/SVG loader helper
processes solely for built-in artwork. Background photo loaders remain available.

Regenerate using `python3 scripts/linux/generate-native-art.py` with ImageMagick
installed. `native-art.json` records source/output hashes and dimensions. These
development tools are not runtime or build dependencies.
