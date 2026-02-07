# EasyAudioFlip

[日本語](README.ja.md)

A system tray app for Windows 11 that lets you switch audio output devices with a single click.

## Features

- **Left-click**: Cycle through the rotation-enabled devices
- **Right-click**: Open the device list panel to toggle which devices are included in the rotation

## Prerequisites

- [mise](https://mise.jdx.dev/) - Tool version manager

## Setup

```bash
mise install
```

## Development

```bash
pnpm tauri dev
```

## Build

```bash
pnpm tauri build
```

The `.msi` installer and `.exe` will be generated in `src-tauri/target/release/bundle/`.

## Release

```bash
node scripts/release.mjs <version>
```

Bumps the version, commits, creates a tag, and pushes in one step. After the push, GitHub Actions builds a Windows installer and creates a release.
