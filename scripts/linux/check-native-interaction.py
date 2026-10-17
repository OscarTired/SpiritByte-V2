#!/usr/bin/env python3
"""Pointer and keyboard regression on an isolated GTK fixture under Hyprland.

Uses compositor input, never Button.clicked(), Entry.set_text(), or the real vault.
Requires an executable compiled with --features ui-check.
"""
import argparse
import json
import os
from pathlib import Path
import signal
import shlex
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("binary", type=Path)
parser.add_argument("--output", type=Path, required=True)
parser.add_argument("--onboarding", action="store_true", help="Exercise the first-run password and language controls")
parser.add_argument("--legacy-overlay", action="store_true", help="Reproduce the former input-blocking overlay (expected failure)")
args = parser.parse_args()
assert os.environ.get("HYPRLAND_INSTANCE_SIGNATURE"), "Hyprland required"
args.output.mkdir(parents=True, exist_ok=True)


lua_mode = False


def hypr(*command):
    global lua_mode
    if lua_mode and command[0] == "dispatch":
        name, value = command[1:]
        q = json.dumps
        if name == "setfloating":
            expression = f"hl.dsp.window.float({{window={q(value)}}})"
        elif name == "resizewindowpixel":
            dimensions, target = value.split(",", 1)
            _, width, height = dimensions.split()
            expression = f"hl.dsp.window.resize({{x={width},y={height},window={q(target)}}})"
        elif name == "centerwindow":
            expression = f"hl.dsp.window.center({{window={q(value)}}})"
        elif name == "focuswindow":
            expression = f"hl.dsp.focus({{window={q(value)}}})"
        elif name == "movecursor":
            x, y = value.split()
            expression = f"hl.dsp.cursor.move({{x={x},y={y}}})"
        elif name == "sendkeystate":
            mods, keyname, state, target = value.split(",", 3)
            expression = f"hl.dsp.send_key_state({{mods={q(mods)},key={q(keyname)},state={q(state)}}})"
        elif name == "sendshortcut":
            mods, keyname, target = value.split(",", 2)
            expression = f"hl.dsp.send_shortcut({{mods={q(mods)},key={q(keyname)},window={q(target)}}})"
        else:
            raise ValueError(name)
        command = ("dispatch", expression)
    result = subprocess.run(["hyprctl", *command], text=True, capture_output=True)
    if result.returncode and not lua_mode and "dispatch in lua" in result.stdout:
        lua_mode = True
        return hypr(*command)
    if result.returncode:
        raise RuntimeError(f"hyprctl {command}: {result.stdout} {result.stderr}")
    return result.stdout


with tempfile.TemporaryDirectory(prefix="spiritbyte-native-check-") as folder:
    base = Path(folder)
    click_tool = base / "native-test-click"
    flags = shlex.split(subprocess.check_output(["pkg-config", "--cflags", "--libs", "wayland-client"], text=True))
    subprocess.run(["cc", "-Wall", "-Wextra", "-o", str(click_tool), str(Path(__file__).with_name("native-test-click.c")), *flags], check=True)
    data = base / ".local/share/com.spiritbyte.app"
    data.mkdir(parents=True)
    state = data / "pointer.json"
    env = dict(os.environ, XDG_DATA_HOME=str(base / ".local/share"),
               XDG_CONFIG_HOME=str(base / ".config"), XDG_CACHE_HOME=str(base / ".cache"),
               SPIRITBYTE_POINTER_CHECK=str(state))
    if args.onboarding:
        env["SPIRITBYTE_AUTH_ONBOARDING"] = "1"
    if args.legacy_overlay:
        env["SPIRITBYTE_TEST_LEGACY_OVERLAY"] = "1"
    with (args.output / "interaction.log").open("w+") as log:
        process = subprocess.Popen([str(args.binary.resolve())], env=env, stdout=log, stderr=log, start_new_session=True)

        def read():
            return json.loads(state.read_text()) if state.exists() else {}

        def wait(predicate, label, seconds=15):
            end = time.monotonic() + seconds
            while time.monotonic() < end:
                current = read()
                if predicate(current):
                    return current
                if process.poll() is not None:
                    break
                time.sleep(.1)
            log.flush()
            raise AssertionError(f"{label}\nState: {read()}\nLog: {(args.output / 'interaction.log').read_text()}")

        def window(title=None):
            clients = json.loads(hypr("clients", "-j"))
            return next(c for c in clients if c["pid"] == process.pid and (title is None or c["title"] == title))

        def node(name=None, kind=None, title="SpiritByte — pointer fixture", index=0):
            snapshot = read()
            w = next(w for w in snapshot["windows"] if w["title"] == title)
            return [n for n in w["widgets"] if (name is None or n["name"] == name) and (kind is None or n["type"] == kind)][index]

        def click(n, title="SpiritByte — pointer fixture", dx=None, dy=None):
            w = window(title)
            target = "address:" + w["address"]
            hypr("dispatch", "focuswindow", target)
            x, y = w["at"]
            hypr("dispatch", "movecursor", f"{x+n['x']+(dx if dx is not None else n['w']//2)} {y+n['y']+(dy if dy is not None else n['h']//2)}")
            time.sleep(.15)
            subprocess.run([str(click_tool)], check=True)
            time.sleep(.2)

        def key(keyname, mods="", title="SpiritByte — pointer fixture"):
            target = "address:" + window(title)["address"]
            hypr("dispatch", "sendshortcut", f"{mods},{keyname},{target}")
            time.sleep(.07)

        def capture(name, title="SpiritByte — pointer fixture"):
            w = window(title)
            hypr("dispatch", "focuswindow", "address:" + w["address"])
            desired = (520, 580) if title == "Entrada de la bóveda" else ((880, 600) if name in ("gtk-compact.png", "gtk-empty.png", "gtk-locked.png") else ((1100, 640) if name in ("gtk-desktop.png", "gtk-intro.png") else None))
            if desired:
                hypr("dispatch", "resizewindowpixel", f"exact {desired[0]} {desired[1]},address:" + w["address"])
            hypr("dispatch", "centerwindow", "address:" + w["address"])
            time.sleep(1)
            w = window(title)
            if desired:
                assert w["size"] == list(desired), f"Capture {name}: requested {desired}, got {w['size']}"
            x, y = w["at"]
            width, height = w["size"]
            (args.output / (name + ".json")).write_text(json.dumps({"client": w, "gtk": read()}, indent=2))
            subprocess.run(["grim", "-g", f"{x},{y} {width}x{height}", str(args.output / name)], check=True)

        def check_social(title="SpiritByte — pointer fixture"):
            for name, url in [("social-github", "https://github.com/OscarTired"), ("social-portfolio", "https://octa-dev.com/"), ("social-paypal", "https://www.paypal.com/paypalme/octacodec")]:
                click(node(name, title=title), title=title)
                wait(lambda s: s.get("last_external_url") == url, "social route " + name)

        def check_auth(registration=False, title="SpiritByte — pointer fixture"):
            fields = ["auth-password", "auth-confirm"] if registration else ["auth-password"]
            for name in fields:
                click(node(name, title=title), title=title)
                for character in "probe":
                    key(character, title=title)
                click(node(name + "-eye", title=title), title=title)
                wait(lambda s: node(name, title=title).get("text") == "probe", "show " + name)
                click(node(name + "-eye", title=title), title=title)
                wait(lambda s: node(name, title=title).get("text") is None, "hide " + name)
            click(node("language-en", title=title), title=title)
            wait(lambda s: s.get("language") == "en", "English button")
            for name in fields:
                click(node(name + "-eye", title=title), title=title)
                wait(lambda s: node(name, title=title).get("text") == "probe", "language preserves " + name)
                click(node(name + "-eye", title=title), title=title)
            click(node("language-es", title=title), title=title)
            wait(lambda s: s.get("language") == "es", "Spanish button")
            check_social(title)

        def check_folder():
            click(node("plus"))
            title = "Nueva carpeta"
            wait(lambda s: any(w["title"] == title for w in s.get("windows", [])), "folder dialog")
            target = "address:" + window(title)["address"]
            hypr("dispatch", "resizewindowpixel", "exact 560 600," + target)
            hypr("dispatch", "centerwindow", target)
            time.sleep(1)
            click(node("folder-name", title=title), title=title)
            for character in "ZZ visual folder":
                key("space" if character == " " else character.lower(), "SHIFT" if character == "Z" else "", title=title)
            click(node("folder-icon-heart", title=title), title=title)
            click(node("folder-color-78c8ff", title=title), title=title)
            capture("gtk-folders.png", title)
            click(node("dialog-save", title=title), title=title)
            wait(lambda s: any(f["name"] == "ZZ visual folder" and f["icon"] == "heart" and f["color"] == "#78c8ff" for f in s.get("folders", [])), "save icon and color")
            created = next(f for f in read()["folders"] if f["name"] == "ZZ visual folder")
            click(node("edit-folder-" + created["id"]))
            title = "Editar carpeta"
            wait(lambda s: any(w["title"] == title for w in s.get("windows", [])), "reopen folder dialog")
            wait(lambda s: node("folder-icon-heart", title=title).get("active") is True and node("folder-color-78c8ff", title=title).get("active") is True, "reload folder appearance")
            assert node("folder-name", title=title)["text"] == "ZZ visual folder"
            capture("gtk-folder-edit.png", title)
            click(node("folder-name", title=title), title=title)
            key("a", "CTRL", title=title)
            for character in "Renamed folder":
                key("space" if character == " " else character.lower(), "SHIFT" if character == "R" else "", title=title)
            click(node("dialog-save", title=title), title=title)
            wait(lambda s: any(f["id"] == created["id"] and f["name"] == "Renamed folder" for f in s["folders"]), "pencil edits same folder")
            assert len(read()["folders"]) == 3
            click(node("plus"))
            title = "Nueva carpeta"
            wait(lambda s: any(w["title"] == title for w in s.get("windows", [])), "plus only creates")
            assert node("folder-name", title=title)["text"] == ""
            key("Escape", title=title)
            wait(lambda s: len(s.get("windows", [])) == 1, "cancel folder edit")

        def check_remove_unfiled():
            click(node("none"))
            wait(lambda s: s.get("filter") == "none", "unfiled filter")
            click(node("remove-unfiled"))
            wait(lambda s: len(s.get("windows", [])) > 1, "remove unfiled confirmation")
            confirm = next(w for w in read()["windows"] if any(n["name"] == "confirmation-accept" for n in w["widgets"]))
            modal = next(c for c in json.loads(hypr("clients", "-j")) if c["pid"] == process.pid and c["title"] != "SpiritByte — pointer fixture")
            click(next(n for n in confirm["widgets"] if n["name"] == "confirmation-accept"), title=modal["title"])
            wait(lambda s: s.get("filter") == "all" and not any(n["name"] == "none" for w in s["windows"] for n in w["widgets"]), "unfiled removed and all selected")
            assert read()["visible_entries"] == 24
            assert json.loads((data / "settings.json").read_text())["showUnfiled"] is False
            # Creating folders must remain available even after hiding the filter.
            assert node("plus")["tooltip"] == "Nueva carpeta"

        try:
            if args.onboarding:
                wait(lambda s: s.get("intro_visible") is True, "fox intro startup")
                target = "address:" + window()["address"]
                if not window()["floating"]:
                    hypr("dispatch", "setfloating", target)
                hypr("dispatch", "resizewindowpixel", "exact 1100 640," + target)
                hypr("dispatch", "centerwindow", target)
                time.sleep(.6)
                capture("gtk-intro.png")
                wait(lambda s: s.get("intro_visible") is False, "intro completes and disposes timer")
                check_auth(registration=True)
                capture("gtk-onboarding.png")
                result = {"backend": "Wayland virtual pointer; Hyprland keyboard events", "checks": ["fox intro and transition", "registration password eye", "confirmation eye", "ES/EN buttons preserve both values", "three social destinations"], "real_vault_accessed": False}
                (args.output / "onboarding.json").write_text(json.dumps(result, indent=2) + "\n")
                print(json.dumps(result, indent=2))
                raise SystemExit(0)
            wait(lambda s: s.get("visible_entries") == 24, "fixture startup", 40)
            client = window("SpiritByte — pointer fixture")
            target = "address:" + client["address"]
            if not client["floating"]:
                hypr("dispatch", "setfloating", target)
            hypr("dispatch", "resizewindowpixel", "exact 1100 640," + target)
            hypr("dispatch", "centerwindow", target)
            time.sleep(1)
            click(node("star"))
            wait(lambda s: s.get("filter") == "star" and s.get("visible_entries") == 6, "click Favorites")
            click(node("child"))
            wait(lambda s: s.get("filter") == "child" and s.get("visible_entries") == 12, "click nested folder")
            click(node("all"))
            wait(lambda s: s.get("visible_entries") == 24, "click All")
            click(node("vault-entries"), dx=60, dy=24)
            wait(lambda s: s.get("selected") is not None, "click credential row")
            check_social()
            check_folder()
            check_remove_unfiled()
            capture("gtk-desktop.png")
            click(node("vault-search"))
            for character in "example 07":
                key("space" if character == " " else character)
            wait(lambda s: s.get("query") == "example 07" and s.get("visible_entries") == 1, "click and type in search")
            key("a", "CTRL")
            key("BackSpace")
            wait(lambda s: s.get("visible_entries") == 24, "clear search with keyboard")
            click(node("new-entry"))
            editor = "Entrada de la bóveda"
            wait(lambda s: any(w["title"] == editor for w in s.get("windows", [])), "click New entry")
            click(node(kind="GtkEntry", title=editor, index=0), title=editor)
            for character in "clicked title":
                key("space" if character == " " else character, title=editor)
            wait(lambda s: any(n.get("text") == "clicked title" for w in s.get("windows", []) if w["title"] == editor for n in w["widgets"]), "click and type in editor")
            capture("gtk-editor.png", editor)
            key("Escape", title=editor)
            wait(lambda s: len(s.get("windows", [])) == 1, "cancel editor")
            click(node("settings"))
            settings = "Ajustes de SpiritByte"
            wait(lambda s: any(w["title"] == settings for w in s.get("windows", [])), "click Settings")
            capture("gtk-settings.png", settings)
            click(node("showUnfiled", title=settings), title=settings)
            click(node("dialog-save", title=settings), title=settings)
            wait(lambda s: len(s.get("windows", [])) == 1 and any(n["name"] == "none" for w in s["windows"] for n in w["widgets"]), "restore unfiled in settings")
            assert json.loads((data / "settings.json").read_text())["showUnfiled"] is True
            hypr("dispatch", "resizewindowpixel", "exact 880 600," + target)
            time.sleep(.5)
            capture("gtk-empty.png")
            click(node("vault-entries"), dx=60, dy=24)
            wait(lambda s: s.get("selected") is not None, "compact credential selection")
            capture("gtk-compact.png")
            click(node("lock"))
            wait(lambda s: s.get("unlocked") is False, "click Lock")
            # Lock clears the main window's credentials and rebuilds the unlock form.
            current = read()
            title = next(w["title"] for w in current["windows"])
            capture("gtk-locked.png", title)
            check_auth(title=title)
            capture("gtk-locked.png", title)
            password = node(kind="GtkEntry", title=title)
            click(password, title=title)
            key("x", title=title)
            wait(lambda s: any(n.get("focus") for w in s.get("windows", []) for n in w["widgets"] if n["type"] == "GtkEntry"), "click locked password field")
            result = {"backend": "Wayland virtual pointer; Hyprland keyboard events", "real_vault_accessed": False,
                      "checks": ["favorites", "nested folder", "all entries", "credential selection", "search focus and typing", "editor focus and typing", "settings", "lock", "unlock field focus", "login eye and ES/EN preserve password", "folder icon/color save and reload", "pencil edits folder; plus creates", "remove unfiled filter preserves entries", "social destinations on vault and login"],
                      "viewports": ["1100x640", "880x600"]}
            (args.output / "interaction.json").write_text(json.dumps(result, indent=2) + "\n")
            print(json.dumps(result, indent=2))
        finally:
            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            process.wait(timeout=5)
