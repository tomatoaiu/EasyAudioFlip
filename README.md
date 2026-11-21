# EasyAudioFlip

[日本語](README.ja.md)

A system tray app for Windows 11 that lets you switch audio output devices with a single click.

![demo](assets/demo.gif)

## Features

- **Left-click**: Cycle through the rotation-enabled devices
- **Right-click**: Open the device list panel to toggle which devices are included in the rotation

## Development

### Prerequisites

- [mise](https://mise.jdx.dev/) - Tool version manager

### Setup

```bash
mise install
```

### Dev

```bash
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

The `.msi` installer and `.exe` will be generated in `src-tauri/target/release/bundle/`.

### Release

Release Please opens a pull request when `main` has releasable commits. Merging that pull request builds the Windows installers and publishes the GitHub release. The next release from `v0.3.3` is `v0.4.0`.
