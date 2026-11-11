# Futjesus Hub

All your Gmail accounts in a single window, Franz style. Built with Tauri 2 and `@konstructio/ui`.

Every account runs in its own webview with an isolated WebKit data store, so sessions never mix. That is why the app requires **macOS 14+**.

## Installation

```sh
brew install --cask futjesus/tap/futjesus-hub
```

Requires macOS 14+ on Apple Silicon. The app is not notarized; the cask removes the quarantine flag so Gatekeeper lets it open.

## Features

- Multiple Gmail accounts, each with its own isolated session
- Inbox unread counter per account and a total badge on the Dock icon
- Profile picture taken from each Gmail account
- Drag to reorder accounts; right click for reload, edit and remove
- Links open in the system browser or inside the app (configurable)
- Light and dark theme
- Memory usage panel and hibernation of idle accounts to free memory

## Shortcuts

| Shortcut | Action |
| --- | --- |
| ⌘1 … ⌘9 | Switch account |
| ⌘⇧R | Reload the current account |
| ⌘= / ⌘- / ⌘0 | Zoom in / out / reset (saved per account) |
| ⌘⇧M | Memory usage |

## Development

```sh
pnpm install
pnpm tauri dev      # development mode
pnpm tauri build    # builds the .app in src-tauri/target/release/bundle
```

Requires Rust (`rustup`) and the Xcode Command Line Tools.

## Structure

- `src-tauri/src/lib.rs`: window, commands and menu.
- `src-tauri/src/gmail.rs`: Gmail webviews, popups, downloads, zoom and the unread/avatar polling.
- `src-tauri/src/status.js`: script that reads the inbox count, email address and profile picture from Gmail's page.
- `src-tauri/src/hibernate.rs`: closes idle accounts and wakes them up on demand.
- `src-tauri/src/memory.rs`: memory usage of each account's WebKit process.
- `src-tauri/src/store.rs`: accounts and settings, saved in `~/Library/Application Support/com.futjesus.hub/accounts.json`.
- `src/`: sidebar and modals in React with `@konstructio/ui`.
- `design/icon.svg`: icon source (`pnpm tauri icon design/icon.svg` regenerates every size).
