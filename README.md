# Futjesus Hub

Todas tus cuentas de Gmail en una sola ventana, al estilo Franz. Hecha con Tauri 2 y `@konstructio/ui`.

Cada cuenta corre en su propio webview con un data store de WebKit aislado, así que las sesiones nunca se mezclan. Por eso la app requiere **macOS 14+**.

## Instalación

```sh
brew install --cask futjesus/tap/futjesus-hub
```

Requiere macOS 14+ en Apple Silicon. La app no está notarizada; el cask le quita la cuarentena para que Gatekeeper la deje abrir.

## Desarrollo

```bash
pnpm install
pnpm tauri dev      # modo desarrollo
pnpm tauri build    # genera .app y .dmg en src-tauri/target/release/bundle
```

Requiere Rust (`rustup`) y las Xcode Command Line Tools.

## Atajos

| Atajo | Acción |
| --- | --- |
| ⌘1 … ⌘9 | Cambiar de cuenta |
| ⌘⇧R | Recargar la cuenta actual |
| ⌘= / ⌘- / ⌘0 | Acercar / alejar / tamaño real (se guarda por cuenta) |

## Estructura

- `src-tauri/src/lib.rs`: ventana, comandos y menú.
- `src-tauri/src/gmail.rs`: webviews de Gmail, ventanas emergentes, descargas, zoom y el sondeo de no leídos/avatar.
- `src-tauri/src/status.js`: script que lee del DOM de Gmail el contador de "Recibidos", el correo y la foto de perfil.
- `src-tauri/src/store.rs`: cuentas y configuración, guardadas en `~/Library/Application Support/com.futjesus.hub/accounts.json`.
- `src/`: barra lateral y modales en React con `@konstructio/ui`.
- `design/icon.svg`: fuente del icono (`pnpm tauri icon design/icon.svg` regenera los tamaños).
