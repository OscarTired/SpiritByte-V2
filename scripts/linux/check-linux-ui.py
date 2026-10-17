#!/usr/bin/env python3
"""Run the native 1,000-entry/GIF regression with isolated XDG data.

Build the linux_ui_check release example first. Accepts a runtime launcher as
its optional first argument, like check-wallpaper-assets.py.
"""
import os
import json
from pathlib import Path
import signal
import shutil
import subprocess
import sys
import tempfile
import time

project = Path(__file__).resolve().parents[2]
binary = Path(sys.argv[1]) if len(sys.argv) > 1 else (
    project / "src-tauri/target/release/examples/linux_ui_check"
)
with tempfile.TemporaryDirectory(prefix="spiritbyte-ui-check-") as folder:
    env = dict(os.environ, XDG_DATA_HOME=folder + "/.local/share",
               XDG_CONFIG_HOME=folder + "/.config", XDG_CACHE_HOME=folder + "/.cache")
    process = subprocess.Popen([str(binary)], env=env, stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT, start_new_session=True)
    try:
        # Verify rendered frames, rather than canvas.drawImage (which uses a
        # GIF's first frame by design). Capture only the disposable test window.
        animation_checked = False
        if os.environ.get("HYPRLAND_INSTANCE_SIGNATURE") and shutil.which("hyprctl") and shutil.which("grim"):
            import cairo
            client = None
            deadline = time.monotonic() + 10
            while time.monotonic() < deadline and process.poll() is None:
                clients = json.loads(subprocess.check_output(["hyprctl", "clients", "-j"]))
                client = next((c for c in clients if c["title"] == "SpiritByte — disposable UI check"), None)
                if client:
                    break
                time.sleep(0.05)
            assert client, "Disposable test window never appeared"
            x, y = client["at"]
            width, height = min(180, client["size"][0]), min(180, client["size"][1])
            colors = set()
            screenshot = Path(folder) / "animation.png"
            for _ in range(30):
                if process.poll() is not None:
                    break
                subprocess.run(["grim", "-g", f"{x},{y} {width}x{height}", str(screenshot)], check=True, capture_output=True)
                surface = cairo.ImageSurface.create_from_png(str(screenshot))
                pixels = surface.get_data()
                for py in range(0, surface.get_height(), 4):
                    for px in range(0, surface.get_width(), 4):
                        i = py * surface.get_stride() + px * 4
                        b, g, r = pixels[i:i + 3]
                        if r > 240 and g < 20 and b < 20:
                            colors.add("red")
                        if b > 240 and g < 20 and r < 20:
                            colors.add("blue")
                if len(colors) == 2:
                    break
                time.sleep(0.05)
            assert colors == {"red", "blue"}, f"GIF did not change rendered frames: {colors}"
            animation_checked = True
        output, _ = process.communicate(timeout=30)
        output = output.decode(errors="replace")
        print(output, flush=True)
        assert process.returncode == 0 and "ui-check:" in output, "Native UI regression failed"
        if animation_checked:
            print("Compositor screenshots verified both animated GIF frames.", flush=True)
    finally:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        process.wait(timeout=5)
print("Native UI passed: GIF loading, wallpaper, CRT effects and search through 1,000 entries.")
