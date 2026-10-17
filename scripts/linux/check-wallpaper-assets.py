#!/usr/bin/env python3
"""Check actual Tauri/WebKit asset decoding without accessing a user's vault.

Build the frontend and `wallpaper_asset_check` release example first.
An optional executable argument supports a bundled runtime launcher.
"""
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile

project = Path(__file__).resolve().parents[2]
binary = Path(sys.argv[1]) if len(sys.argv) > 1 else (
    project / "src-tauri/target/release/examples/wallpaper_asset_check"
)

for name, args, expected in [
    ("previous scope", ["--old-scope"], False),
    ("fixed scope", [], True),
]:
    with tempfile.TemporaryDirectory(prefix="spiritbyte-asset-check-") as folder:
        env = dict(os.environ, XDG_DATA_HOME=folder + "/.local/share",
                   XDG_CONFIG_HOME=folder + "/.config", XDG_CACHE_HOME=folder + "/.cache")
        process = subprocess.Popen([str(binary), *args], env=env, stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT, start_new_session=True)
        try:
            output, _ = process.communicate(timeout=25)
            output = output.decode(errors="replace")
            print(name, flush=True)
            print(output, flush=True)
            rows = [line for line in output.splitlines() if line.startswith("decoder:")]
            assert len(rows) == 5, "The native runtime did not check all five formats"
            assert all(("loaded=true" in row) == expected for row in rows), output
            assert process.returncode == (0 if expected else 1), output
        finally:
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            process.wait(timeout=5)

print("Native regression passed: old scope fails; fixed scope decodes all five formats.")
