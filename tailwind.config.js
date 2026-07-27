/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        bg: "rgb(var(--sb-bg) / <alpha-value>)",
        surface: "rgb(var(--sb-surface) / <alpha-value>)",
        "surface-2": "rgb(var(--sb-surface-2) / <alpha-value>)",
        border: "rgb(var(--sb-border) / <alpha-value>)",
        text: "rgb(var(--sb-text) / <alpha-value>)",
        "text-dim": "rgb(var(--sb-text-dim) / <alpha-value>)",
        primary: "rgb(var(--sb-primary) / <alpha-value>)",
        accent: "rgb(var(--sb-accent) / <alpha-value>)",
        danger: "rgb(var(--sb-danger) / <alpha-value>)",
        success: "rgb(var(--sb-success) / <alpha-value>)",
        warning: "rgb(var(--sb-warning) / <alpha-value>)",
      },
      fontFamily: {
        pixel: ["'Press Start 2P'", "monospace"],
        mono: ["'VT323'", "ui-monospace", "monospace"],
      },
      fontSize: {
        term: ["1em", { lineHeight: "1.3" }],
        xs: ["0.72em", { lineHeight: "1.4" }],
        sm: ["0.82em", { lineHeight: "1.25" }],
        base: ["1em", { lineHeight: "1.5" }],
        lg: ["1.12em", { lineHeight: "1.75" }],
        xl: ["1.25em", { lineHeight: "1.75" }],
      },
      boxShadow: {
        glow: "0 0 8px rgb(var(--sb-primary) / 0.5), 0 0 2px rgb(var(--sb-primary) / 0.8)",
        "glow-accent": "0 0 8px rgb(var(--sb-accent) / 0.5)",
      },
      keyframes: {
        flicker: {
          "0%, 100%": { opacity: "1" },
          "92%": { opacity: "1" },
          "93%": { opacity: "0.82" },
          "94%": { opacity: "1" },
        },
        blink: {
          "0%, 49%": { opacity: "1" },
          "50%, 100%": { opacity: "0" },
        },
      },
      animation: {
        flicker: "flicker 6s linear infinite",
        blink: "blink 1s step-end infinite",
      },
    },
  },
  plugins: [],
};
