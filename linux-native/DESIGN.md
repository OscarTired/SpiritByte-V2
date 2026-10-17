---
name: SpiritByte — native Linux GTK
description: Existing customizable retro CRT and pixel identity, implemented with native GTK 3 widgets.
colors:
  primary: "rgb(255, 176, 0)"
  bg: "rgb(13, 12, 10)"
  surface: "rgb(26, 23, 18)"
  surface2: "rgb(38, 33, 24)"
  border: "rgb(82, 66, 38)"
  text: "rgb(245, 214, 150)"
  textDim: "rgb(168, 140, 92)"
  danger: "rgb(240, 92, 64)"
typography:
  title:
    fontFamily: '"Press Start 2P", monospace'
    fontSize: "15px"
  body:
    fontFamily: '"VT323", monospace'
    fontSize: "18px"
  label:
    fontFamily: '"Press Start 2P", monospace'
    fontSize: "10px"
rounded:
  square: "0px"
spacing:
  tight: "4px"
  field-gap: "6px"
  compact: "8px"
  field-inset: "12px"
  section-gap: "16px"
  panel-inset: "24px"
components:
  button:
    backgroundColor: "{colors.surface2}"
    textColor: "{colors.text}"
    rounded: "{rounded.square}"
    padding: "6px 10px"
  input:
    backgroundColor: "{colors.bg}"
    textColor: "{colors.text}"
    rounded: "{rounded.square}"
    padding: "4px 10px"
  detail-field:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    padding: "{spacing.field-inset}"
---

# Design System: SpiritByte — native Linux GTK

## Overview

**Creative North Star: "SpiritByte retro/CRT/pixel"**

This documents the shipped GTK 3 client only. Its established terminal-like palette, pixel headings and compact native controls follow the Windows reference without creating a new identity. Read [PRODUCT.md](../PRODUCT.md) for product truth and scope. The Windows client remains its own visual authority; this file does not govern or authorize changes to it or the shared core.

Native implementation truth: [main.rs](src/main.rs) (`apply_settings`, root draw effects and settings dialog), [ui.rs](src/ui.rs) (vault composition and icons), [model.rs](src/model.rs) (defaults and persistence), [palettes.json](assets/palettes.json), [fonts.rs](src/fonts.rs) and [font asset notes](assets/README.md). Read-only Windows references: [VaultApp.tsx](../src/screens/VaultApp.tsx), [index.css](../src/index.css), [settings.ts](../src/theme/settings.ts) and [palettes.ts](../src/theme/palettes.ts). The reviewed native [desktop](../.impeccable/review/gtk-desktop.png) and [compact](../.impeccable/review/gtk-compact.png) captures illustrate this implementation; source governs adjustable values.

**Key Characteristics:**
- Three native resizable panes with independent vertical scrolling.
- Square outlined controls, dark tonal surfaces and restrained phosphor glow.
- User-adjustable palettes, fonts, backgrounds and CRT effects.
- Native Rust/GTK rendering with low-resource behavior.

## Colors

The frontmatter records the **default Amber CRT** rendering, not a fixed palette. `paletteId` and the full role map persist in native settings. Green Phosphor, Synthwave 84, IBM Noir, Commodore 64 and custom colors remain supported; take exact preset values from `assets/palettes.json`. Palette roles `accent`, `success` and `warning` are stored too, but do not currently define recurring GTK control styles, so they are not promoted to extracted visual tokens here.

### Primary

The primary phosphor role marks branding, titles, focus, selected content and emphasized actions. Selection uses the current primary at 12% alpha rather than a separate fixed hue. Danger marks destructive-action hover.

### Neutral

`bg` supports the window and input interiors; `surface` supports dialogs and panels; `surface2` supports buttons and hover rows. `border` separates panes and frames fields. `text` carries readable values; `textDim` carries captions, counts and status. Panel surfaces use the user-selected `panelOpacity` (default fully opaque).

**The Palette Role Rule.** Derive native styles from the persisted palette roles; Amber CRT values are defaults, not hardcoded replacements for user choices.

## Typography

The default mixed mode pairs VT323 body text with Press Start 2P branding, titles, buttons and captions. The frontmatter describes that mode. Branding uses a smaller pixel size (12px); counts and list status use a compact size (14px). Pixel-font buttons use the rounded product of configured size and 0.61 (11px at the default).

Configured base size is adjustable (10–24px). Mono mode uses VT323; pixel mode uses Press Start 2P with body size rounded from base × 0.67. The five Geist Pixel modes use Square, Grid, Circle, Triangle or Line with body size rounded from base × 0.78. Display roles use the selected family outside mixed/pixel mode; title, brand and caption sizes remain as implemented in `apply_settings`.

**The Real Font Rule.** Embed and register the seven shipped TTF assets with process-local Fontconfig/Pango registration. Preserve the existing WOFF2 sources and licenses; do not rely on WOFF2 registration alone or install fonts globally.

## Layout

The vault uses nested horizontal `GtkPaned` widgets: navigation, searchable entry list, then detail. Initial sidebar and list positions are 224px and 288px; their minimum requested widths are 192px and 240px. Splitters remain draggable. The default window is 1100 × 720, with a minimum request of 880 × 600, in GTK logical pixels. Compact desktop windows retain the three-pane structure; no mobile breakpoint is implemented.

Navigation, entries and detail each have native vertical scrollers with horizontal scrolling disabled. Long list titles ellipsize; detail titles and values wrap. Headers and footers use compact insets, detail uses panel insets, and field groups use section gaps. Nested folder indentation advances by 12px, capped at five levels. Entry cells use 12px horizontal and 9px vertical padding.

## Elevation & Depth

The native UI uses tonal layering and borders, with no button or input box shadows. Optional primary-colored text glow (0 0 3px) accents the brand and title. Optional scanlines draw one dark pixel row every four logical pixels at 13% alpha. Optional flicker is a full-window dark pulse at 6% opacity for one 500ms phase every two seconds; it is disabled by default.

**The Existing Draw Rule.** Paint CRT effects after content on the existing root overlay. Keep icon drawing areas windowless so decoration cannot intercept clicks. GIFs invalidate changed rectangles; the optional flicker clock checks at 500ms and requests a full repaint only when the global pulse changes. Low-power mode pauses animation while the window is inactive; unmapped windows do not run animation.

GIF backgrounds use an incremental Rust LZW decoder and native rectangular pixel compositing/disposal, retaining a canvas, the current patch and an optional disposal rectangle instead of a full animation cache. Painting is capped at 20 FPS; intervening source frames update disposal state without being painted. Preserve source timing and loop behavior, bounded decode allocations, and the existing single rendered-surface cache.

## Shapes

Controls, field frames, panels, tabs, selection rows and scrollbar sliders use square geometry. Panels, detail fields, inputs and regular buttons have two-pixel borders; pane separators are four logical pixels wide and highlight with primary on hover. These are GTK content rules, not a requirement for window-manager decorations. Cairo icons inherit state color; their line glyphs use rounded line caps and joins.

## Components

- **Buttons:** default tonal fill and square outline; hover changes the border and child text/icon to primary. Active and primary variants use the translucent selection fill. Focus uses a one-pixel primary inset outline (offset −4px); disabled controls use 40% opacity. Destructive hover uses danger.
- **Icon buttons:** windowless 16px Cairo glyphs inside native buttons, transparent at rest with 4px padding and 20px minimum content dimensions. Keep localized tooltips and accessible names on the button; the brand glyph is 18px.
- **Inputs:** dark interiors, square outlines and primary caret/focus border. Use actual GTK entry, search, text and spin widgets so selection, typing and keyboard focus remain native.
- **Navigation and entry rows:** hover uses `surface2`; selection uses primary tint and primary foreground. Navigation retains folder counts and indentation. The native entry tree supports multiple selection; row activation edits an entry.
- **Detail fields:** framed tonal groups with uppercase captions, selectable wrapped values and adjacent copy actions. Passwords use a masked, noneditable native entry with explicit reveal; empty-value copy actions are disabled.
- **Folder appearance:** the folder dialog offers the same 22 icon IDs as Windows, eight preset colors, the active palette color, and a native custom color chooser. The saved icon and hex color render in navigation and remain part of the existing encrypted folder record.
- **Folder actions:** the section's plus button opens a blank creation form. Each saved folder has a visible pencil that opens its own name, parent, icon and color. Only the edit form offers deletion, preserving entries and subfolders. The automatic “Sin carpeta” filter has a remove action that hides it from the sidebar without deleting entries; Settings restores it. Folder creation remains available when the folder list is empty.
- **Startup intro:** reuse the Windows fox SVG through its checked-in RGBA raster, Bayer dithering, orbital marks, scan reveal and boot lines. Cache the 420px artwork once, paint it at 320 logical pixels, finish after 3.2 seconds and release the artwork and timer. Honor GTK's animation preference, the saved palette, and `showSplash`; pause background animation during the intro. It runs only at startup, not on each lock. Built-in raw fox/icon assets avoid persistent image-loader helpers solely for startup artwork; their manifest records the unchanged Windows source hashes.
- **Authentication:** master-password and confirmation fields have separate keyboard-focusable eye buttons. ES/EN buttons switch language immediately while preserving typed values and returning fields to their masked state.
- **Profile links:** GitHub, portfolio and PayPal icons appear in the authentication card and at the bottom right of the vault. Use the URLs confirmed in PRODUCT.md, with localized tooltips and accessible names.
- **Settings and empty states:** preserve native notebook tabs and scrolled settings content. Empty details use the key glyph and localized instruction rather than fabricated data. Appearance changes save through the existing native settings flow.

## Do's and Don'ts

### Do:
- **Do** preserve the established retro identity and use Windows as the read-only visual reference.
- **Do** resolve palette and font choices from persisted native preferences.
- **Do** retain native pane resizing, scrolling, text selection and keyboard focus.
- **Do** verify visual changes with the GTK client and isolated fixture data.

### Don't:
- **Don't** modify the Windows implementation or shared core to implement native visual refinements.
- **Don't** add an input-catching decoration window over interactive content.
- **Don't** introduce a web runtime or redundant animation clock into the native client.
- **Don't** treat the default amber palette or mixed font mode as the only supported appearance.
