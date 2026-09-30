<div align="center">

# SpiritByte

### Local-first retro password manager

A secure, fully customizable password manager with a retro CRT/pixel aesthetic. Built with Tauri 2 (Rust) + React/TypeScript. Your data never leaves your device.

[![Tauri](https://img.shields.io/badge/Tauri-2.x-orange?logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-1.77+-red?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18-blue?logo=react&logoColor=white)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-blue?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

[Features](#features) &middot; [Backups](#encrypted-backups) &middot; [Performance](#performance) &middot; [Security](#security-model) &middot; [Install](#installation) &middot; [Build](#building-from-source) &middot; [Architecture](#architecture)

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
- Selectable notes with a one-click copy-all button
- Password-protected encrypted vault import/export from Settings
- Configurable password generator with entropy strength meter
- Auto-lock on inactivity and automatic clipboard clearing

**Customization**
- Multiple preset palettes + custom color editor (live preview)
- Retro typography (Press Start 2P, VT323, Geist Pixel) with adjustable font size
- Backgrounds: solid, gradient, or custom image
- CRT effects: scanlines, glow, flicker, and Bayer 8x8 dithered splash screen

**Cross-platform**
- Native installers for Windows (MSI via WiX, NSIS), macOS, and Linux
- Android version available: [SpiritByte for Android](https://github.com/OscarTired/SpiritByte-Android), built with native Kotlin/Compose and the shared Rust vault engine
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

## Encrypted backups

Choose **All** to export the entire vault, or **Select** to choose folders and individual credentials. Checking a folder selects its contents and subfolders; uncheck any individual credentials to exclude them. Required parent folders, icons and colors are preserved automatically. Explicitly selected empty folders can also be exported. **Clear** resets the selection, and an empty selection cannot be exported.

In **Settings > Import / Export vault**, choose a long, unique backup password (at least 12 characters), use the strength meter as guidance, confirm it, and export to a new `.spiritbyte` file using the native save dialog. The live character counter and matching-password indicator show when export is enabled; the strength estimate is advisory. Keep the password separately: the vault recovery phrase cannot unlock this backup. Existing files are never overwritten.

To restore, create or unlock a destination vault, select the backup in Settings, enter its backup password, and choose **Import and add**. All entries (including notes, favorites, timestamps and icons), folders and parent relationships are preserved. Imported records receive fresh IDs, so existing records are retained; importing twice creates duplicates. Visual preferences, wallpapers, master credentials and recovery phrases are not included. The destination vault keeps its own credentials.

Format v1 uses fixed Argon2id parameters (64 MiB, 3 iterations, 1 lane), a fresh random salt, and XChaCha20-Poly1305 authenticated encryption with a fresh nonce. No decrypted backup is written to disk. Imports are limited to 32 MiB and authenticated and structurally validated before merging. The encrypted vault is synced to a sibling temporary file and atomically replaced before the session is updated.

## Performance

- Search sorting and normalized text are cached until entries change, avoiding repeated sorting while typing or switching folders.
- Entry and folder saves update the affected frontend records without fetching the entire vault again. Unchanged entry rows reuse their rendered output.
- Password strength checks are debounced, and the inactivity timer tracks activity without recreating a timeout on every mouse movement.
- The splash precomputes its shaded Bayer-dithered fox once, then reveals the cached artwork. Its vector facets, subtle phosphor glow and scan sweep follow the active palette; reduced-motion preferences skip the sweep.
- Pending saves and refreshes cannot repopulate the frontend after locking. Vault writes use atomic replacement, and failed saves retain the previous data.

## Installation

### Pre-built binaries

Download the latest installer from the [Releases](https://github.com/OscarTired/SpiritByte-V2/releases) page:

| Platform | Installer |
|---|---|
| Windows | `.msi` (WiX) or `-setup.exe` (NSIS) |
| macOS | `.dmg` |
| Linux | `.deb` / `.AppImage` |
| Android | See [SpiritByte-Android](https://github.com/OscarTired/SpiritByte-Android) for downloads and build instructions |

### Linux / CachyOS

For CachyOS and other Arch-based distributions, use the **x86_64 AppImage** on an Intel/AMD laptop. The `.deb` package is intended for Debian/Ubuntu.

Copy the AppImage to your laptop, then make it executable and launch it:

```bash
chmod +x SpiritByte_*.AppImage
./SpiritByte_*.AppImage
```

If launching reports a missing `libfuse.so.2`, install FUSE 2 with `sudo pacman -Syu fuse2`, or run the AppImage with `--appimage-extract-and-run`.

### Updating on Windows

For version 0.1.1, close SpiritByte and run the new installer using the same installer format as your current installation (MSI or NSIS EXE). Install over the existing version; there is no need to uninstall first. The application identifier and MSI upgrade code remain unchanged, and the vault stays in the existing app-data directory. The `src-tauri/target` directory contains build artifacts only and is regenerated when building.

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

Tauri builds bundles for the current operating system. On Windows, `bundle.targets: "all"` produces MSI/NSIS installers; Linux packages must be built in a Linux environment. See the [Tauri AppImage guide](https://v2.tauri.app/distribute/appimage/) for portability requirements.

**Build Linux packages from Windows with Docker Desktop**

Start Docker Desktop with Linux containers enabled, then run from the desktop project in PowerShell. The Rust core is included in this repository; no Android checkout is required:

```powershell
./scripts/build-linux.ps1
```

The builder uses Ubuntu 22.04, installs Linux dependencies, and builds the current local sources. It writes `.AppImage` files to `src-tauri/target/release/bundle/appimage/` and `.deb` files to `src-tauri/target/release/bundle/deb/`, alongside the Windows installers. The first build downloads the toolchains; subsequent builds reuse Docker's image and Rust caches.

**Build directly on CachyOS / Arch Linux**

Install Node.js, pnpm, a current Rust toolchain, and the [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/). Clone the desktop repository before building:

```bash
git clone https://github.com/OscarTired/SpiritByte-V2.git
cd SpiritByte-V2
pnpm install --frozen-lockfile
pnpm tauri build --bundles appimage -- --locked
```

The resulting AppImage is in `src-tauri/target/release/bundle/appimage/`. Builds made on a rolling distribution target that system's libraries; use the Ubuntu Docker builder for distribution to older Linux systems.

**Run crypto tests**

```bash
cargo test --manifest-path core/Cargo.toml --locked
```

The vault engine is the standalone `spiritbyte-core` Rust library in this
repository's `core/` directory. Desktop builds use only this local copy and do
not require the Android repository or Android SDK. Android includes its own copy
of the same library so it can also build independently.

`src-tauri/src/{crypto,vault,backup,generator}.rs` are compatibility re-exports.
Make engine changes in `core/src`. When changing the engine, apply the same
source and dependency changes to Android's `core/` and run the core tests in both
repositories to preserve vault and backup compatibility. Copies do not synchronize
automatically. See [the core library README](core/README.md).

## Architecture

```
src/                       Frontend (React + TypeScript)
  components/              UI primitives, SplashFox, StrengthMeter
  features/                generator, entries, settings
  screens/                 Onboarding, Unlock, VaultApp
  store/                   Zustand state (useVault, useSettings)
  theme/                   Palettes + theme application (CSS vars)
  lib/                     API bridge, clipboard, auto-lock, types

core/                      Standalone spiritbyte-core library (included locally)
  src/crypto.rs            Argon2id, XChaCha20-Poly1305, BIP39
  src/vault.rs             Data model, on-disk format, operations
  src/backup.rs            Encrypted portable backups and additive restore
  src/generator.rs         Password generator + strength estimation
  tests/                   Export selection integration tests

src-tauri/src/             Desktop backend (Rust)
  {crypto,vault,backup,generator}.rs  Core compatibility re-exports
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

- [x] Password-protected encrypted vault import/export (`.spiritbyte`)
- [x] Search, rendering, clipboard feedback and inactivity-timer improvements
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
