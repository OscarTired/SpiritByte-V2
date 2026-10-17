#!/usr/bin/env python3
"""Exercise the opt-in native GTK UI check on disposable data and verify GIF frames.
Build: cargo build --manifest-path linux-native/Cargo.toml --release --features ui-check
Never run this against the user's vault. Requires Hyprland/grim for rendered QA.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('binary',type=Path)
parser.add_argument('--output',type=Path)
args=parser.parse_args()
project=Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix='spiritbyte-native-check-') as folder:
    base=Path(folder)
    data=base/'.local/share/com.spiritbyte.app'
    data.mkdir(parents=True)
    shutil.copyfile(project/'src-tauri/examples/fixtures/wallpaper.animated.gif',data/'wallpaper.check.gif')
    report=data/'ui-check.json'
    env=dict(os.environ,XDG_DATA_HOME=str(base/'.local/share'),XDG_CONFIG_HOME=str(base/'.config'),XDG_CACHE_HOME=str(base/'.cache'),SPIRITBYTE_NATIVE_CHECK=str(report))
    with (base/'runtime.log').open('w+') as log:
        process=subprocess.Popen([str(args.binary.resolve())],env=env,stdout=log,stderr=log,start_new_session=True)
        try:
            deadline=time.monotonic()+40
            while not report.exists() and time.monotonic()<deadline and process.poll() is None:
                time.sleep(.1)
            if not report.exists():
                log.seek(0)
                runtime_log=log.read()
                if args.output:
                    args.output.parent.mkdir(parents=True,exist_ok=True)
                    args.output.with_suffix('.log').write_text(runtime_log)
                raise RuntimeError('Native UI check failed: '+runtime_log)
            result=json.loads(report.read_text())
            assert os.environ.get('HYPRLAND_INSTANCE_SIGNATURE'), 'Hyprland required for actual rendered frame check'
            import cairo
            clients=json.loads(subprocess.check_output(['hyprctl','clients','-j']))
            client=next(c for c in clients if c['pid']==process.pid)
            x,y=client['at'];w,h=client['size']
            screenshot=base/'frame.png'
            colors=set()
            for _ in range(40):
                subprocess.run(['grim','-g',f'{x+w-min(w,160)},{y+h-min(h,160)} {min(w,160)}x{min(h,160)}',str(screenshot)],check=True,capture_output=True)
                surface=cairo.ImageSurface.create_from_png(str(screenshot));pixels=surface.get_data()
                for py in range(0,surface.get_height(),4):
                    for px in range(0,surface.get_width(),4):
                        i=py*surface.get_stride()+px*4;b,g,r=pixels[i:i+3]
                        if r>235 and g<25 and b<25:colors.add('red')
                        if b>235 and g<25 and r<25:colors.add('blue')
                if len(colors)==2:break
                time.sleep(.07)
            assert colors=={'red','blue'},f'GIF did not change rendered frames: {colors}'
            result['gif_rendered_frames']=sorted(colors)
            if args.output:
                args.output.parent.mkdir(parents=True,exist_ok=True)
                args.output.write_text(json.dumps(result,indent=2)+'\n')
                subprocess.run(['grim','-g',f'{x},{y} {w}x{h}',str(args.output.with_suffix('.png'))],check=True,capture_output=True)
            log.seek(0);runtime_log=log.read()
            assert 'CRITICAL' not in runtime_log and 'panicked' not in runtime_log,runtime_log
            print(json.dumps(result,indent=2))
            if runtime_log:print(runtime_log)
        finally:
            try:os.killpg(process.pid,signal.SIGTERM)
            except ProcessLookupError:pass
            process.wait(timeout=5)
