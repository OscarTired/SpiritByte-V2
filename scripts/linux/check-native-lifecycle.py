#!/usr/bin/env python3
"""Measure repeated native screen disposal on an isolated ui-check fixture."""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("binary", type=Path)
parser.add_argument("--output", type=Path, required=True)
parser.add_argument("--assert-stable", action="store_true")
args = parser.parse_args()
args.output.parent.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix="spiritbyte-native-check-") as directory:
    base = Path(directory)
    data = base / ".local/share/com.spiritbyte.app"
    data.mkdir(parents=True)
    (data / "settings.json").write_text(json.dumps({"showSplash": False}))
    report = data / "stress.json"
    env = dict(os.environ, XDG_DATA_HOME=str(base / ".local/share"),
               XDG_CONFIG_HOME=str(base / ".config"), XDG_CACHE_HOME=str(base / ".cache"),
               SPIRITBYTE_POINTER_CHECK=str(report), SPIRITBYTE_STRESS_CHECK="1")
    with args.output.with_suffix(".log").open("w+") as log:
        process = subprocess.Popen([str(args.binary.resolve())], env=env, stdout=log, stderr=log, start_new_session=True)
        try:
            assert process.wait(timeout=55) == 0
            log.seek(0)
            runtime = log.read()
            assert "CRITICAL" not in runtime and "panicked" not in runtime, runtime
            result = json.loads(report.read_text())
            samples = result["samples"]
            result["growth_after_warmup_mib"] = round((samples[-1]["memory_kib"]["Pss"] - samples[1]["memory_kib"]["Pss"]) / 1024, 2)
            args.output.write_text(json.dumps(result, indent=2) + "\n")
            print(json.dumps(result, indent=2))
            if args.assert_stable:
                assert samples[-1]["retained_discarded_entries"] == 0, "Discarded password widgets are still alive"
                assert result["growth_after_warmup_mib"] < 8, "Repeated navigation keeps growing memory"
        finally:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGTERM)
                process.wait(timeout=5)
