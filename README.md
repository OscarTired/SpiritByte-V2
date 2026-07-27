<div align="center">

# SpiritByte

### Local-first retro password manager

A secure, fully customizable password manager with a retro CRT/pixel aesthetic. Built with Tauri 2 (Rust) + React/TypeScript. Your data never leaves your device.

[![Tauri](https://img.shields.io/badge/Tauri-2.x-orange?logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.77+-red?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18-blue?logo=react&logoColor=white)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-blue?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

[Features](#features) &middot; [Security](#security-model) &middot; [Install](#installation) &middot; [Build](#building-from-source) &middot; [Architecture](#architecture)

</div>

---

## Screenshots

<table>
  <tr>
    <td width="50%" align="center"><b>MSI Installer (WiX)</b></td>
    <td width="50%" align="center"><b>Onboarding — Master Key Setup</b></td>
  </tr>
  <tr>
    <td><img src="Ss-readme/Install.png" alt="SpiritByte MSI installer with custom branding" /></td>
    <td><img src="Ss-readme/create-master-key.png" alt="Onboarding screen showing master key creation with strength meter" /></td>
  </tr>
</table>

<details>
<summary>Theme customization</summary>

<table>
  <tr>
    <td width="33%" align="center"><b>Solid</b></td>
    <td width="33%" align="center"><b>Gradient</b></td>
    <td width="33%" align="center"><b>Image / GIF</b></td>
  </tr>
  <tr>
    <td><img src="Ss-readme/Theme-solid.png" alt="Solid background theme" /></td>
    <td><img src="Ss-readme/Theme-gradient.png" alt="Gradient background theme" /></td>
    <td><img src="Ss-readme/Theme-image&gif.png" alt="Image and GIF background theme" /></td>
  </tr>
</table>

</details>

## Features

**Cryptography**
- Argon2id key derivation + XChaCha20-Poly1305 authenticated encryption
- Envelope encryption: a random DEK encrypts the vault, wrapped twice (master password + BIP39 recovery phrase)
- 12-word BIP39 recovery phrase for full access recovery
- All secrets are zeroized in memory — the DEK and master password never leave the Rust process

**Vault Management**
- Entries with title, username, password, URL, and notes
- Nested folders with custom icons and colors
- Favorites, full-text search, and keyboard-driven workflow
- Configurable password generator with entropy strength meter
- Auto-lock on inactivity and automatic clipboard clearing

**Customization**
- Multiple preset palettes + custom color editor (live preview)
- Retro typography (Press Start 2P, VT323, Geist Pixel) with adjustable font size
- Backgrounds: solid, gradient, or custom image
- CRT effects: scanlines, glow, flicker, and Bayer 8x8 dithered splash screen

**Cross-platform**
- Native installers for Windows (MSI via WiX, NSIS), macOS, and Linux
- Multilingual: English and Spanish with runtime switching

> [!TIP]
> SpiritByte stores everything locally in your OS app-data directory. No servers, no accounts, no telemetry. Your encrypted vault is the only artifact that matters.

## Security Model

| Artifact | Contents | Encryption |
|---|---|---|
| `vault.dat` | Entries and folders | XChaCha20-Poly1305 (DEK) |
| `vault.meta.json` | Salts, Argon2 params, wrapped DEK (x2) | No plaintext secrets |
| `settings.json` | UI preferences | Plaintext (non-sensitive) |

```
Master Password ──┐
                  ├──(Argon2id)──> DEK ──> XChaCha20-Poly1305 ──> vault.dat
Recovery Phrase ──┘
```

> [!WARNING]
> If you lose **both** your master password **and** your 12-word recovery phrase, the vault is unrecoverable by design. There is no backdoor.

## Installation

### Pre-built binaries

Download the latest installer from the [Releases](https://github.com/OscarTired/SpiritByte-V2/releases) page:

| Platform | Installer |
|---|---|
| Windows | `.msi` (WiX) or `-setup.exe` (NSIS) |
| macOS | `.dmg` |
| Linux | `.deb` / `.AppImage` |

### Building from source

**Prerequisites**

- [Node.js](https://nodejs.org/) 18+ and [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install) toolchain (`cargo`)
- Platform-specific Tauri [prerequisites](https://v2.tauri.app/start/prerequisites/)

**Development**

```bash
pnpm install
pnpm tauri dev
```

**Production build**

```bash
pnpm tauri build
```

**Run crypto tests**

```bash
cd src-tauri
cargo test --lib
```

## Architecture

```
src/                       Frontend (React + TypeScript)
  components/              UI primitives, SplashFox, StrengthMeter
  features/                generator, entries, settings
  screens/                 Onboarding, Unlock, VaultApp
  store/                   Zustand state (useVault, useSettings)
  theme/                   Palettes + theme application (CSS vars)
  lib/                     API bridge, clipboard, auto-lock, types

src-tauri/src/             Backend (Rust)
  crypto.rs                Argon2id, XChaCha20-Poly1305, BIP39
  vault.rs                 Data model, on-disk format, operations
  generator.rs             Password generator + strength estimation
  state.rs                 Session state (DEK in memory)
  commands.rs              Tauri IPC commands exposed to frontend
```

<details>
<summary>Tech stack</summary>

| Layer | Technology |
|---|---|
| Desktop runtime | Tauri 2 |
| Backend | Rust (argon2, chacha20poly1305, bip39, zeroize) |
| Frontend | React 18, TypeScript 5.6 |
| State | Zustand 5 |
| Styling | Tailwind CSS 3.4 |
| Icons | Lucide React |
| Fonts | Press Start 2P, VT323, Geist Pixel (5 variants) |

</details>

## Roadmap

- [ ] CSV import/export
- [ ] Browser extension autofill

## Contributing

Contributions are welcome. Please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Commit your changes
4. Open a pull request

## Support

If you find SpiritByte useful, consider supporting the project:

[![PayPal](https://img.shields.io/badge/PayPal-Donate-blue?logo=paypal&logoColor=white)](https://paypal.me/octacodec)
[![GitHub](https://img.shields.io/badge/GitHub-Sponsor-purple?logo=github&logoColor=white)](https://github.com/OscarTired)
[![Website](https://img.shields.io/badge/Web-octa--dev.com-teal)](https://octa-dev.com/)

## License

[MIT](LICENSE)
