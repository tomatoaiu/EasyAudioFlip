# EasyAudioFlip

Windows 11 のタスクトレイに常駐し、音声出力デバイスをワンクリックで切り替えるアプリ。

![demo](assets/demo.gif)

## 機能

- **左クリック**: ローテーション対象のデバイスを順番に切り替え
- **右クリック**: デバイス一覧パネルを表示し、ローテーション対象のオン/オフを切り替え

## 開発

### 必要環境

- [mise](https://mise.jdx.dev/) - ツールバージョン管理

### セットアップ

```bash
mise install
```

### 開発サーバー

```bash
pnpm tauri dev
```

### ビルド

```bash
pnpm tauri build
```

`src-tauri/target/release/bundle/` に `.msi` インストーラーと `.exe` が生成されます。

### リリース

`main` にリリース対象のコミットがあると、Release Please が pull request を作る。その pull request をマージすると、Windows 向けインストーラーをビルドして GitHub Release を公開する。`v0.3.3` の次は `v0.4.0`。
