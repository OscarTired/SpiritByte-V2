# SpiritByte

Gestor de contraseñas **local, cifrado de extremo a extremo** y **full customizable** con estética retro CRT/pixel. Construido con **Tauri 2 (Rust)** + **React/TypeScript** (pnpm).

## Características

- **Cifrado fuerte:** Argon2id (derivación de clave) + XChaCha20-Poly1305 (cifrado del vault).
- **Envelope encryption:** una DEK aleatoria cifra el vault y se "envuelve" dos veces — con la contraseña maestra y con una frase de recuperación BIP39 de 12 palabras.
- **Recuperación por 12 palabras:** permite recuperar el acceso y resetear la contraseña maestra.
- **Local-first:** todo se guarda cifrado en el directorio de datos de la app. La DEK y la contraseña maestra nunca salen del proceso Rust (zeroized en memoria).
- **Gestión:** entradas (título, usuario, contraseña, URL, notas), carpetas anidadas, favoritos, búsqueda.
- **Generador de contraseñas** configurable con medidor de fuerza (entropía).
- **Auto-lock** por inactividad y **limpieza automática del portapapeles**.
- **Personalización:** paletas predefinidas + editor de color, tipografías retro, fondos (sólido/gradiente/imagen), toggles de scanlines, glow y flicker.
- **Splash "Wireframe Fox"** con efecto dither Bayer 8x8 y secuencia de boot retro.

## Requisitos

- Node.js + `pnpm`
- Rust toolchain (`cargo`)

## Desarrollo

```bash
pnpm install
pnpm tauri dev
```

## Build de producción

```bash
pnpm tauri build
```

## Tests del core de cifrado

```bash
cd src-tauri
cargo test --lib
```

## Arquitectura

```
src/                      Frontend React + TypeScript
  components/             UI primitives, SplashFox, StrengthMeter
  features/               generator, entries, settings
  screens/                Onboarding, Unlock, VaultApp
  store/                  Zustand (useVault, useSettings)
  theme/                  paletas + aplicación de tema (CSS vars)
  lib/                    api (invoke), clipboard, auto-lock, tipos
src-tauri/src/            Backend Rust
  crypto.rs               Argon2id, XChaCha20-Poly1305, BIP39
  vault.rs                modelo de datos, formato en disco, operaciones
  generator.rs            generador de contraseñas + fuerza
  state.rs                estado de sesión (DEK en memoria)
  commands.rs             comandos Tauri expuestos al frontend
```

## Modelo de seguridad

| Artefacto | Contenido | Cifrado |
|-----------|-----------|---------|
| `vault.dat` | entradas y carpetas | XChaCha20-Poly1305 con la DEK |
| `vault.meta.json` | salts, params Argon2, DEK envuelta (x2) | sin secretos en claro |
| `settings.json` | preferencias de UI | claro (no sensible) |

> **Importante:** si pierdes la contraseña maestra **y** la frase de 12 palabras, el vault es irrecuperable por diseño.
