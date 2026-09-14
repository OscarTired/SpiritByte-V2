import { useEffect, useRef, useState } from "react";
import { useSettings } from "@/store/useSettings";
import { useI18n } from "@/lib/i18n";

/** Faceted phosphor fox. Build the dithered artwork once, then reveal a cached
 * layer with a scan sweep instead of processing every source pixel per frame. */

// 8x8 Bayer threshold matrix (0..63 normalized to 0..1).
const BAYER_8 = [
  [0, 32, 8, 40, 2, 34, 10, 42],
  [48, 16, 56, 24, 50, 18, 58, 26],
  [12, 44, 4, 36, 14, 46, 6, 38],
  [60, 28, 52, 20, 62, 30, 54, 22],
  [3, 35, 11, 43, 1, 33, 9, 41],
  [51, 19, 59, 27, 49, 17, 57, 25],
  [15, 47, 7, 39, 13, 45, 5, 37],
  [63, 31, 55, 23, 61, 29, 53, 21],
];

const BOOT_LINES = [
  "SPIRITBYTE SYSTEM v0.1.1",
  "SECURITY CORE ........ READY",
  "ENCRYPTION LAYER ..... ACTIVE",
  "ZERO-KNOWLEDGE ....... VERIFIED",
  "SECURE STORAGE ....... MOUNTED",
  "VAULT SHIELD ......... ENGAGED",
  "PHOSPHOR FOX ......... ONLINE",
];

function channelToColor(channel: string): string {
  const [r, g, b] = channel.trim().split(/\s+/).map((n) => parseInt(n, 10));
  return `rgb(${r}, ${g}, ${b})`;
}

interface SplashFoxProps {
  onDone: () => void;
  duration?: number;
}

export function SplashFox({ onDone, duration = 3200 }: SplashFoxProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const palette = useSettings((s) => s.settings.palette);
  const ditherIntensity = useSettings((s) => s.settings.ditherIntensity);
  const glow = useSettings((s) => s.settings.glow);
  const doneRef = useRef(onDone);
  useEffect(() => { doneRef.current = onDone; }, [onDone]);
  const t = useI18n((s) => s.t);
  const [bootIndex, setBootIndex] = useState(0);
  const [fading, setFading] = useState(false);

  // Boot log typing.
  useEffect(() => {
    setBootIndex(0);
    setFading(false);
    const step = duration / (BOOT_LINES.length + 2);
    const timers = BOOT_LINES.map((_, i) =>
      window.setTimeout(() => setBootIndex(i + 1), step * (i + 1)),
    );
    const fadeT = window.setTimeout(() => setFading(true), duration - 450);
    const doneT = window.setTimeout(() => doneRef.current(), duration);
    return () => {
      timers.forEach(clearTimeout);
      clearTimeout(fadeT);
      clearTimeout(doneT);
    };
  }, [duration]);

  useEffect(() => {
    let raf = 0;
    let cancelled = false;
    const controller = new AbortController();
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    const W = 420;
    canvas.width = canvas.height = W;
    const scale = Math.round(Math.max(1, Math.min(4, ditherIntensity || 2)));
    const primary = channelToColor(palette.primary);
    const accent = channelToColor(palette.accent);
    const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const source = document.createElement("canvas");
    source.width = source.height = W;
    const sourceCtx = source.getContext("2d", { willReadFrequently: true });
    const artwork = document.createElement("canvas");
    artwork.width = artwork.height = W;
    const art = artwork.getContext("2d");
    if (!sourceCtx || !art) return;

    async function loadFox() {
      const response = await fetch("/fox-byte.svg", { signal: controller.signal });
      if (!response.ok) throw new Error("Fox artwork unavailable");
      const svg = await response.text();
      // Retain the original facets. Bright seams separate the shaded planes.
      const styled = svg.replace(/<svg([^>]*)>/i, `<svg$1><style>path{stroke:#fff;stroke-width:2.2;stroke-linejoin:round;stroke-opacity:.85}</style>`);
      const url = URL.createObjectURL(new Blob([styled], { type: "image/svg+xml" }));
      try {
        const img = new Image();
        await new Promise<void>((resolve, reject) => {
          img.onload = () => resolve();
          img.onerror = () => reject(new Error("Fox artwork unavailable"));
          img.src = url;
        });
        return img;
      } finally { URL.revokeObjectURL(url); }
    }

    function buildArtwork(img: HTMLImageElement | null) {
      if (!sourceCtx || !art) return;
      if (img) sourceCtx.drawImage(img, 30, 30, 360, 360);
      else {
        // A small vector fallback keeps the splash recognizable if asset loading fails.
        sourceCtx.strokeStyle = "white";
        sourceCtx.lineWidth = 2;
        sourceCtx.fillStyle = "#b36a28";
        const outline = new Path2D("M105 85 L180 140 L240 140 L315 85 L292 240 L210 325 L128 240 Z M105 85 L155 220 L210 325 L265 220 L315 85 M155 220 L185 230 M265 220 L235 230");
        sourceCtx.fill(outline);
        sourceCtx.stroke(outline);
      }
      const pixels = sourceCtx.getImageData(0, 0, W, W).data;
      // Average each block so thin ears, whiskers and seams survive coarse dithering.
      for (let y = 0; y < W; y += scale) {
        for (let x = 0; x < W; x += scale) {
          let intensity = 0;
          let peak = 0;
          let samples = 0;
          for (let dy = 0; dy < scale && y + dy < W; dy++) {
            for (let dx = 0; dx < scale && x + dx < W; dx++) {
              const i = ((y + dy) * W + x + dx) * 4;
              const value = (pixels[i] * .299 + pixels[i + 1] * .587 + pixels[i + 2] * .114) / 255 * pixels[i + 3] / 255;
              intensity += value;
              peak = Math.max(peak, value);
              samples++;
            }
          }
          intensity /= samples;
          const threshold = BAYER_8[(y / scale) & 7][(x / scale) & 7] / 64;
          if (intensity > .03 && intensity > threshold * .85) {
            art.fillStyle = peak > .78 ? accent : primary;
            art.globalAlpha = peak > .78 ? .95 : .35 + intensity * .55;
            art.fillRect(x, y, scale, scale);
          }
        }
      }
      art.globalAlpha = 1;
    }

    function drawFrame(reveal: number) {
      if (!ctx) return;
      ctx.clearRect(0, 0, W, W);
      // Broken orbital guides and registration marks frame the asymmetric fox.
      ctx.strokeStyle = primary;
      ctx.lineWidth = 1;
      ctx.globalAlpha = .16;
      ctx.beginPath();
      ctx.arc(210, 210, 190, .15, 1.30);
      ctx.arc(210, 210, 190, 1.72, 2.98);
      ctx.arc(210, 210, 190, 3.30, 4.50);
      ctx.arc(210, 210, 190, 4.88, 6.10);
      ctx.stroke();
      ctx.globalAlpha = .4;
      for (const [x, y, sx, sy] of [[37, 37, 1, 1], [383, 37, -1, 1], [37, 383, 1, -1], [383, 383, -1, -1]]) {
        ctx.beginPath();
        ctx.moveTo(x, y + sy * 15); ctx.lineTo(x, y); ctx.lineTo(x + sx * 15, y);
        ctx.stroke();
      }
      ctx.globalAlpha = 1;
      ctx.save();
      ctx.beginPath(); ctx.rect(0, 0, W, W * reveal); ctx.clip();
      if (glow) {
        ctx.shadowColor = primary;
        ctx.shadowBlur = 10;
        ctx.globalAlpha = .45;
        ctx.drawImage(artwork, 0, 0);
        ctx.shadowBlur = 0;
        ctx.globalAlpha = 1;
      }
      ctx.drawImage(artwork, 0, 0);
      ctx.restore();
      if (reveal < 1) {
        const y = W * reveal;
        const beam = ctx.createLinearGradient(35, 0, 385, 0);
        beam.addColorStop(0, "transparent"); beam.addColorStop(.5, accent); beam.addColorStop(1, "transparent");
        ctx.fillStyle = beam;
        ctx.globalAlpha = .65;
        ctx.fillRect(35, y, 350, 1);
        ctx.globalAlpha = 1;
      }
    }

    const started = performance.now();
    async function render() {
      let img: HTMLImageElement | null = null;
      try { img = await loadFox(); } catch { /* Use vector fallback. */ }
      if (cancelled) return;
      buildArtwork(img);
      const animate = (now: number) => {
        if (cancelled) return;
        const reveal = reducedMotion ? 1 : Math.min(1, (now - started) / Math.max(1, duration * .65));
        drawFrame(reveal);
        if (reveal < 1) raf = requestAnimationFrame(animate);
      };
      raf = requestAnimationFrame(animate);
    }
    void render();
    return () => { cancelled = true; controller.abort(); cancelAnimationFrame(raf); };
  }, [ditherIntensity, palette.primary, palette.accent, glow, duration]);

  return (
    <div
      className={`fixed inset-0 z-[100] flex flex-col items-center justify-center bg-bg transition-opacity duration-500 ${
        fading ? "opacity-0" : "opacity-100"
      }`}
    >
      <canvas
        role="img"
        aria-label="SpiritByte fox"
        ref={canvasRef}
        className="image-render-pixel"
        style={{ imageRendering: "pixelated", width: "min(420px, 54vh, 90vw)", height: "min(420px, 54vh, 90vw)" }}
      />
      <h1 className="font-pixel text-primary text-shadow-glow text-xl mt-2 tracking-widest">
        SPIRITBYTE
      </h1>
      <p className="text-text-dim text-term mt-1">{t("splash.tagline")}</p>

      <div className="mt-6 h-44 shrink-0 w-[min(420px,90vw)] overflow-y-auto font-mono text-base text-success/90">
        {BOOT_LINES.slice(0, bootIndex).map((line) => (
          <div key={line} className="leading-tight">
            <span className="text-text-dim">&gt;</span> {line}
          </div>
        ))}
        {bootIndex < BOOT_LINES.length && (
          <span className="inline-block w-3 h-4 bg-primary animate-blink" />
        )}
      </div>
    </div>
  );
}
