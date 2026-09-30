import type { Palette } from "./palettes";
import type { Settings } from "./settings";
import { backgroundSource } from "./background";

let activeSettings: Settings | undefined;
let listening = false;

function applyBackground(settings: Settings) {
  const body = document.body;
  const { background } = settings;
  let value = "";
  if (background.type === "image" && background.value) {
    // Removing the image while in the background stops GIF decoding/rendering.
    if (!settings.lowPowerMode || (!document.hidden && document.hasFocus())) {
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
    window.addEventListener("focus", refresh);
    window.addEventListener("blur", refresh);
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
