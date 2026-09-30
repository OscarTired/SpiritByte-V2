import { test } from "node:test";
import assert from "node:assert/strict";
import ts from "typescript";
import { readFile } from "node:fs/promises";

const source = await readFile(new URL("../src/theme/applyTheme.ts", import.meta.url), "utf8");
const { outputText } = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2020 },
});
let instance = 0;
async function theme() {
  const events = {};
  const classes = {};
  const assignments = [];
  const style = new Proxy({ backgroundImage: "" }, {
    set(target, key, value) {
      if (key === "backgroundImage") assignments.push(value);
      target[key] = value;
      return true;
    },
  });
  let focused = true;
  globalThis.document = {
    hidden: false,
    hasFocus: () => focused,
    documentElement: { style: { setProperty() {} }, classList: { toggle(key, value) { classes[key] = value; } }, dataset: {} },
    body: { style },
    addEventListener(event, handler) { events[event] = handler; },
  };
  globalThis.window = { addEventListener(event, handler) { events[event] = handler; } };
  const code = outputText.replace('import { backgroundSource } from "./background";',
    'const backgroundSource = (value) => `asset://localhost/${encodeURIComponent(value)}`;');
  const { applyTheme } = await import(`data:text/javascript;base64,${Buffer.from(code).toString("base64")}#${instance++}`);
  return { applyTheme, style, classes, assignments, events, focus(value) { focused = value; } };
}

const settings = {
  palette: {}, background: { type: "image", value: "/test/wallpaper.first.gif" },
  scanlines: true, glow: true, flicker: true, lowPowerMode: false,
  font: "mono", fontSize: 18, panelOpacity: 1,
};

test("unrelated settings reuse the same background without reloading the GIF", async () => {
  const t = await theme();
  t.applyTheme(settings);
  const original = t.style.backgroundImage;
  t.applyTheme({ ...settings, fontSize: 20 });
  assert.equal(t.style.backgroundImage, original);
  assert.equal(t.assignments.length, 1);
  assert.ok(!original.includes("?t="));
  t.applyTheme({ ...settings, background: { type: "image", value: "/test/wallpaper.second.gif" } });
  assert.equal(t.assignments.length, 2);
});

test("resource saving stops backgrounds on blur or hide and restores on focus", async () => {
  const t = await theme();
  t.applyTheme({ ...settings, lowPowerMode: true });
  const original = t.style.backgroundImage;
  assert.equal(t.classes["fx-flicker"], false);
  t.focus(false);
  t.events.blur();
  assert.equal(t.style.backgroundImage, "");
  t.focus(true);
  t.events.focus();
  assert.equal(t.style.backgroundImage, original);
  document.hidden = true;
  t.events.visibilitychange();
  assert.equal(t.style.backgroundImage, "");
  document.hidden = false;
  t.events.visibilitychange();
  assert.equal(t.style.backgroundImage, original);
  t.applyTheme(settings);
  t.focus(false);
  t.events.blur();
  assert.equal(t.style.backgroundImage, original);
  assert.equal(t.classes["fx-flicker"], true);
});
