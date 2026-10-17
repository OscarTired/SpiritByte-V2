#!/usr/bin/env python3
"""Verify the production native client's GIF redraws without a second CRT timer."""
import argparse,json,os,shutil,signal,subprocess,tempfile,time
from pathlib import Path
import cairo
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('binary',type=Path)
p.add_argument('--output',type=Path)
a=p.parse_args()
project=Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix='spiritbyte-native-gif-') as tmp:
    base=Path(tmp);data=base/'.local/share/com.spiritbyte.app';data.mkdir(parents=True)
    gif=data/'wallpaper.gif';shutil.copyfile(project/'src-tauri/examples/fixtures/wallpaper.animated.gif',gif)
    (data/'settings.json').write_text(json.dumps({'background':{'type':'image','value':str(gif)},'lowPowerMode':False,'flicker':True,'scanlines':True,'glow':True}))
    env=dict(os.environ,XDG_DATA_HOME=str(base/'.local/share'),XDG_CONFIG_HOME=str(base/'.config'),XDG_CACHE_HOME=str(base/'.cache'))
    with (base/'runtime.log').open('w+') as log:
        app=subprocess.Popen([str(a.binary.resolve())],env=env,stdout=log,stderr=log,start_new_session=True)
        try:
            deadline=time.monotonic()+15;client=None
            while time.monotonic()<deadline and app.poll() is None:
                client=next((c for c in json.loads(subprocess.check_output(['hyprctl','clients','-j'])) if c['pid']==app.pid),None)
                if client:break
                time.sleep(.1)
            if not client:
                log.seek(0);raise RuntimeError('No native test window: '+log.read())
            time.sleep(2)
            x,y=client['at'];w,h=client['size'];colors=set();shot=base/'gif.png'
            for _ in range(50):
                subprocess.run(['grim','-g',f'{x},{y} {min(w,160)}x{min(h,160)}',str(shot)],check=True,capture_output=True)
                surface=cairo.ImageSurface.create_from_png(str(shot));pixels=surface.get_data()
                for py in range(0,surface.get_height(),4):
                    for px in range(0,surface.get_width(),4):
                        i=py*surface.get_stride()+px*4;b,g,r=pixels[i:i+3]
                        if r>235 and g<25 and b<25:colors.add('red')
                        if b>235 and g<25 and r<25:colors.add('blue')
                if len(colors)==2:break
                time.sleep(.07)
            assert colors=={'red','blue'},f'Production GIF stalled: {colors}'
            result={'production_gif_rendered_frames':sorted(colors),'flicker':True,'lowPowerMode':False}
            if a.output:a.output.write_text(json.dumps(result,indent=2)+'\n')
            print(json.dumps(result,indent=2))
        finally:
            try:os.killpg(app.pid,signal.SIGTERM)
            except ProcessLookupError:pass
            app.wait(timeout=5)
