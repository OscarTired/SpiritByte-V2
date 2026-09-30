# spiritbyte-core

Standalone Rust library for SpiritByte's encryption, vault persistence,
encrypted backups and password generation. It has no Tauri, Android, JNI or
UniFFI dependencies and can be built directly from this directory:

```sh
cargo build --locked
cargo test --locked
```

Desktop and Android each include this crate as `core/` in their own repository.
The desktop Tauri crate and Android native bridge depend on their local copy via
`path = "../core"`. Building either application does not require the other
repository. This library is compiled into the application's binary.

The copies are maintained manually. Apply engine source and dependency changes
to both copies, keep the crate version aligned, and run the tests in both
repositories. Changes to vault or backup formats must preserve compatibility
between desktop and Android. Do not modify only a platform's compatibility
re-exports to change engine behavior.

Licensed under the [MIT License](LICENSE).
