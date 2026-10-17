import type { Palette } from "./palettes";
import type { Settings } from "./settings";
import { backgroundSource } from "./background";

let activeSettings: Settings | undefined;
let listening = false;
// WebKitGTK can report hasFocus() = false during startup on Wayland. Only
// suspend animations after an actual blur, otherwise the first wallpaper vanishes.
let focused = true;

function applyBackground(settings: Settings) {
  const body = document.body;
  const { background } = settings;
  let value = "";
  if (background.type === "image" && background.value) {
    const animated = /\.(gif|webp)$/i.test(background.value) ||
      /^data:image\/(gif|webp)[;,]/i.test(background.value);
    // Keep photos visible in inactive windows. Only unload animated formats on
    // blur; unload all images when hidden to release decoded surfaces.
    if (!settings.lowPowerMode || (!document.hidden && (focused || !animated))) {
      value = `url(${JSON.stringify(backgroundSource(background.value))})`;
    }
  } else if (background.type === "gradient" && background.value.includes("|")) {
    const [a, b] = background.value.split("|");
    value = `linear-gradient(135deg, ${a}, ${b})`;
  }
  // Stable file names change on import; unrelated settings reuse the same URL.
  if (body.style.backgroundImage !== value) body.style.backgroundImage = value;
  body.style.backgroundSize = "cover";
  body.style.backgroundPosition = "center";
}

const VAR_MAP: Record<keyof Palette, string> = {
  bg: "--sb-bg",
  surface: "--sb-surface",
  surface2: "--sb-surface-2",
  border: "--sb-border",
  text: "--sb-text",
  textDim: "--sb-text-dim",
  primary: "--sb-primary",
  accent: "--sb-accent",
  danger: "--sb-danger",
  success: "--sb-success",
  warning: "--sb-warning",
};

export function applyTheme(settings: Settings) {
  activeSettings = settings;
  if (!listening) {
    const refresh = () => {
      if (activeSettings) applyBackground(activeSettings);
    };
    document.addEventListener("visibilitychange", refresh);
    window.addEventListener("focus", () => { focused = true; refresh(); });
    window.addEventListener("blur", () => { focused = false; refresh(); });
    listening = true;
  }
  const root = document.documentElement;
  const { palette } = settings;
  (Object.keys(VAR_MAP) as (keyof Palette)[]).forEach((key) => {
    root.style.setProperty(VAR_MAP[key], palette[key]);
  });

  root.classList.toggle("fx-scanlines", settings.scanlines);
  root.classList.toggle("fx-glow", settings.glow);
  root.classList.toggle("fx-flicker", settings.flicker && !settings.lowPowerMode);

  root.dataset.font = settings.font;

  root.style.setProperty("--sb-panel-opacity", String(settings.panelOpacity));
  root.style.setProperty("--sb-font-size", `${settings.fontSize}px`);

  applyBackground(settings);
}

/** Returns a CSS rgb() string for a palette channel. */
export function rgb(channel: string, alpha = 1): string {
  return alpha === 1 ? `rgb(${channel})` : `rgb(${channel} / ${alpha})`;
}
