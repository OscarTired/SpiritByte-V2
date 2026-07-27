import { useEffect, useRef, useState } from "react";
import { useSettings } from "@/store/useSettings";
import { useI18n } from "@/lib/i18n";

/**
 * Boot splash: rasterizes fox-byte.svg as a WIREFRAME (fill:none, stroke only),
 * then applies an 8x8 ordered (Bayer) dither so the fox renders as retro dots
 * in the active phosphor color. A scanline sweep progressively reveals it,
 * accompanied by a fake boot log.
 */

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
  "SPIRITBYTE SYSTEM v0.1.0",
  "SECURITY CORE ........ READY",
  "ENCRYPTION LAYER ..... ACTIVE",
  "ZERO-KNOWLEDGE ....... VERIFIED",
  "SECURE STORAGE ....... MOUNTED",
  "VAULT SHIELD ......... ENGAGED",
  "WIREFRAME FOX ........ ONLINE",
];

function channelToHex(channel: string): string {
  const [r, g, b] = channel.trim().split(/\s+/).map((n) => parseInt(n, 10));
  return `rgb(${r}, ${g}, ${b})`;
}

interface SplashFoxProps {
  onDone: () => void;
  duration?: number;
}

export function SplashFox({ onDone, duration = 3200 }: SplashFoxProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { settings } = useSettings();
  const t = useI18n((s) => s.t);
  const [bootIndex, setBootIndex] = useState(0);
  const [fading, setFading] = useState(false);

  // Boot log typing.
  useEffect(() => {
    const step = duration / (BOOT_LINES.length + 2);
    const timers = BOOT_LINES.map((_, i) =>
      window.setTimeout(() => setBootIndex(i + 1), step * (i + 1)),
    );
    const fadeT = window.setTimeout(() => setFading(true), duration - 450);
    const doneT = window.setTimeout(onDone, duration);
    return () => {
      timers.forEach(clearTimeout);
      clearTimeout(fadeT);
      clearTimeout(doneT);
    };
  }, [duration, onDone]);

  // Render wireframe + dither.
  useEffect(() => {
    let raf = 0;
    let cancelled = false;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const scale = Math.max(1, Math.min(4, settings.ditherIntensity || 2));
    const primary = channelToHex(settings.palette.primary);
    const accent = channelToHex(settings.palette.accent);

    const W = 420;
    const H = 420;
    canvas.width = W;
    canvas.height = H;

    const off = document.createElement("canvas");
    off.width = W;
    off.height = H;
    const offCtx = off.getContext("2d", { willReadFrequently: true });
    if (!offCtx) return;

    async function buildWireframeImage(): Promise<HTMLImageElement | null> {
      try {
        const res = await fetch(`/fox-byte.svg?t=${Date.now()}`);
        let svg = await res.text();
        // Force wireframe: no fills, white strokes, drop gradients.
        const style = `<style>*{fill:none !important;stroke:#fff !important;stroke-width:3px !important;}</style>`;
        svg = svg.replace(/<defs[\s\S]*?<\/defs>/i, "");
        svg = svg.replace(/<svg([^>]*)>/i, `<svg$1>${style}`);
        const blob = new Blob([svg], { type: "image/svg+xml" });
        const url = URL.createObjectURL(blob);
        return await new Promise((resolve) => {
          const img = new Image();
          img.onload = () => {
            URL.revokeObjectURL(url);
            resolve(img);
          };
          img.onerror = () => {
            URL.revokeObjectURL(url);
            resolve(null);
          };
          img.src = url;
        });
      } catch {
        return null;
      }
    }

    // The source pixels are static across frames, so rasterize the wireframe
    // and read its pixels a single time, then reuse them every frame.
    function rasterizeSource(img: HTMLImageElement | null): Uint8ClampedArray | null {
      if (!offCtx) return null;
      offCtx.clearRect(0, 0, W, H);
      if (img) {
        // Fit image preserving aspect.
        const pad = 40;
        const size = Math.min(W, H) - pad * 2;
        offCtx.drawImage(img, pad, pad, size, size);
      }
      return offCtx.getImageData(0, 0, W, H).data;
    }

    function ditherFrame(src: Uint8ClampedArray, reveal: number) {
      if (!ctx) return;

      ctx.clearRect(0, 0, W, H);
      const revealY = reveal * H;

      for (let y = 0; y < H; y += scale) {
        if (y > revealY) break;
        for (let x = 0; x < W; x += scale) {
          // Average luminance/alpha of the block.
          const idx = (y * W + x) * 4;
          const alpha = src[idx + 3] / 255;
          const lum =
            (src[idx] * 0.299 + src[idx + 1] * 0.587 + src[idx + 2] * 0.114) /
            255;
          const intensity = alpha * lum;
          const threshold = BAYER_8[(y / scale) % 8 | 0][(x / scale) % 8 | 0] / 64;
          if (intensity > threshold && intensity > 0.04) {
            ctx.fillStyle = intensity > 0.6 ? accent : primary;
            ctx.fillRect(x, y, scale, scale);
          }
        }
      }

      // Scanline sweep highlight.
      if (reveal < 1) {
        ctx.fillStyle = `${accent}`;
        ctx.globalAlpha = 0.5;
        ctx.fillRect(0, revealY - scale, W, scale * 2);
        ctx.globalAlpha = 1;
      }
    }

    const start = performance.now();
    buildWireframeImage().then((img) => {
      if (cancelled) return;
      const src = rasterizeSource(img);
      if (!src) return;
      const animate = (t: number) => {
        const reveal = Math.min(1, (t - start) / (duration * 0.7));
        ditherFrame(src, reveal);
        if (reveal < 1 && !cancelled) {
          raf = requestAnimationFrame(animate);
        }
      };
      raf = requestAnimationFrame(animate);
    });

    return () => {
      cancelled = true;
      cancelAnimationFrame(raf);
    };
  }, [settings.ditherIntensity, settings.palette]);

  return (
    <div
      className={`fixed inset-0 z-[100] flex flex-col items-center justify-center bg-bg transition-opacity duration-500 ${
        fading ? "opacity-0" : "opacity-100"
      }`}
    >
      <canvas
        ref={canvasRef}
        className="image-render-pixel"
        style={{ imageRendering: "pixelated", width: 420, height: 420 }}
      />
      <h1 className="font-pixel text-primary text-shadow-glow text-xl mt-2 tracking-widest">
        SPIRITBYTE
      </h1>
      <p className="text-text-dim text-term mt-1">{t("splash.tagline")}</p>

      <div className="mt-6 h-40 w-[420px] overflow-y-auto font-mono text-term text-success/90">
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
