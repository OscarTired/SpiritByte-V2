#!/usr/bin/env python3
"""Measure a release executable/AppImage with disposable settings and no real vault.

Reports PSS (shared pages apportioned), summed RSS, and idle CPU for the entire
process tree. Compare runs on the same desktop, screen and wallpaper.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import statistics
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("executable", type=Path)
parser.add_argument("--wallpaper", type=Path)
parser.add_argument("--flicker", action="store_true")
parser.add_argument("--native", action="store_true", help="Require a native client without WebKit subprocesses")
parser.add_argument("--verify-fixture-gif", action="store_true", help="After sampling, verify red/blue frames of the two-frame test GIF on Hyprland")
parser.add_argument("--warmup", type=int, default=15)
parser.add_argument("--seconds", type=int, default=20)
parser.add_argument("--output", type=Path)
parser.add_argument("--size", help="Fix the Hyprland window size, e.g. 1100x720, for comparable measurements")
args = parser.parse_args()
assert args.warmup >= 0 and args.seconds > 0


def snapshot(root):
    processes = {}
    for stat in Path("/proc").glob("[0-9]*/stat"):
        try:
            raw = stat.read_text()
            fields = raw[raw.rfind(")") + 2:].split()
            processes[int(stat.parent.name)] = (int(fields[1]), int(fields[11]) + int(fields[12]))
        except (OSError, ValueError, IndexError):
            continue
    tree = {root}
    while True:
        children = {pid for pid, (parent, _) in processes.items() if parent in tree}
        if children <= tree:
            break
        tree |= children
    rows = []
    for pid in sorted(tree):
        try:
            values = {}
            for line in Path(f"/proc/{pid}/smaps_rollup").read_text().splitlines():
                parts = line.split()
                if len(parts) == 3:
                    values[parts[0].rstrip(":")] = int(parts[1])
            rows.append({"pid": pid, "name": Path(f"/proc/{pid}/comm").read_text().strip(),
                         "pss_mib": values["Pss"] / 1024,
                         "rss_mib": values["Rss"] / 1024,
                         "ticks": processes[pid][1]})
        except (OSError, KeyError):
            continue
    return rows


with tempfile.TemporaryDirectory(prefix="spiritbyte-memory-") as folder:
    base = Path(folder)
    data = base / ".local/share/com.spiritbyte.app"
    data.mkdir(parents=True)
    settings = {"showSplash": True, "scanlines": True, "glow": True,
                "flicker": args.flicker, "lowPowerMode": False, "panelOpacity": 0.85,
                "background": {"type": "solid", "value": ""}}
    if args.wallpaper:
        wallpaper = data / ("wallpaper.measure" + args.wallpaper.suffix.lower())
        shutil.copyfile(args.wallpaper, wallpaper)
        settings["background"] = {"type": "image", "value": str(wallpaper)}
    (data / "settings.json").write_text(json.dumps(settings))
    env = dict(os.environ, XDG_DATA_HOME=str(base / ".local/share"),
               XDG_CONFIG_HOME=str(base / ".config"), XDG_CACHE_HOME=str(base / ".cache"))
    with (base / "runtime.log").open("w+") as log:
        process = subprocess.Popen([str(args.executable.resolve())], env=env,
                                   stdout=log, stderr=log, start_new_session=True)
        try:
            if args.size:
                width, height = map(int, args.size.split("x"))
                time.sleep(1)
                clients=json.loads(subprocess.check_output(["hyprctl","clients","-j"]))
                client=next(c for c in clients if c["pid"]==process.pid)
                target="address:"+client["address"]
                for command,value,expression in [
                    ("setfloating",target,f"hl.dsp.window.float({{window={json.dumps(target)}}})"),
                    ("resizewindowpixel",f"exact {width} {height},{target}",f"hl.dsp.window.resize({{x={width},y={height},window={json.dumps(target)}}})"),
                    ("centerwindow",target,f"hl.dsp.window.center({{window={json.dumps(target)}}})"),
                ]:
                    if command=="setfloating" and client["floating"]:continue
                    attempt=subprocess.run(["hyprctl","dispatch",command,value],capture_output=True)
                    if attempt.returncode:
                        subprocess.run(["hyprctl","dispatch",expression],check=True,capture_output=True)
            time.sleep(args.warmup)
            samples = []
            start = time.monotonic()
            first = snapshot(process.pid)
            for _ in range(args.seconds):
                if process.poll() is not None:
                    log.seek(0)
                    raise RuntimeError(f"Runtime exited ({process.returncode}): {log.read()}")
                time.sleep(1)
                samples.append(snapshot(process.pid))
            elapsed = time.monotonic() - start
            final = samples[-1]
            webkit = any(row["name"].startswith("WebKit") for row in final)
            if args.native and webkit:
                raise RuntimeError(f"Native client unexpectedly started WebKit: {final}")
            if not args.native and not any(row["name"].startswith("WebKitWeb") for row in final):
                log.seek(0)
                raise RuntimeError(f"No measurable WebKit renderer found. Processes: {final}\n{log.read()}")
            ticks_before = {row["pid"]: row["ticks"] for row in first}
            cpu_ticks = sum(max(0, row["ticks"] - ticks_before.get(row["pid"], row["ticks"])) for row in final)
            rendered_frames = None
            if args.verify_fixture_gif:
                import cairo
                clients = json.loads(subprocess.check_output(["hyprctl", "clients", "-j"]))
                client = next(c for c in clients if c["pid"] == process.pid)
                x, y = client["at"]
                width, height = client["size"]
                colors = set()
                shot = base / "fixture-gif.png"
                for _ in range(40):
                    subprocess.run(["grim", "-g", f"{x},{y} {min(width,160)}x{min(height,160)}", str(shot)], check=True, capture_output=True)
                    surface = cairo.ImageSurface.create_from_png(str(shot))
                    pixels = surface.get_data()
                    for py in range(0, surface.get_height(), 4):
                        for px in range(0, surface.get_width(), 4):
                            i = py * surface.get_stride() + px * 4
                            b, g, r = pixels[i:i + 3]
                            if r > 235 and g < 25 and b < 25:
                                colors.add("red")
                            if b > 235 and g < 25 and r < 25:
                                colors.add("blue")
                    if len(colors) == 2:
                        break
                    time.sleep(0.07)
                assert colors == {"red", "blue"}, f"GIF stopped animating after warmup/sample: {colors}"
                rendered_frames = sorted(colors)
            geometry=None
            if os.environ.get("HYPRLAND_INSTANCE_SIGNATURE"):
                client=next(c for c in json.loads(subprocess.check_output(["hyprctl","clients","-j"])) if c["pid"]==process.pid)
                geometry={"logical_size":client["size"],"floating":client["floating"]}
                if args.size:assert client["size"]==[width,height],geometry
            result = {"executable": str(args.executable.resolve()), "native": args.native, "screen": "onboarding; disposable XDG data; no real vault", "window_geometry":geometry,
                      "runtime_overrides": {key: env[key] for key in ("JSC_useJIT", "MALLOC_ARENA_MAX", "GDK_BACKEND", "WEBKIT_DISABLE_COMPOSITING_MODE", "WEBKIT_DISABLE_DMABUF_RENDERER") if key in env},
                      "settings": settings, "warmup_seconds": args.warmup, "sample_seconds": round(elapsed, 2),
                      "pss_mib": round(statistics.mean(sum(row["pss_mib"] for row in sample) for sample in samples), 1),
                      "rss_sum_mib": round(statistics.mean(sum(row["rss_mib"] for row in sample) for sample in samples), 1),
                      "cpu_one_core_percent": round(cpu_ticks / os.sysconf("SC_CLK_TCK") / elapsed * 100, 2),
                      "gif_rendered_after_sample": rendered_frames,
                      "processes": [dict({k: round(v, 1) if isinstance(v, float) else v for k, v in row.items() if k != "ticks"},
                                         cpu_one_core_percent=round(max(0, row["ticks"]-ticks_before.get(row["pid"],row["ticks"])) / os.sysconf("SC_CLK_TCK") / elapsed * 100, 2)) for row in final]}
            print(json.dumps(result, indent=2), flush=True)
            if args.output:
                args.output.write_text(json.dumps(result, indent=2) + "\n")
        finally:
            # Stop only the test processes that this script created.
            rows = snapshot(process.pid)
            for row in reversed(rows):
                try:
                    os.kill(row["pid"], signal.SIGTERM)
                except ProcessLookupError:
                    pass
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=5)
