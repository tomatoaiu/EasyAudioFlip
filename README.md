# EasyAudioFlip

Windows 11 のタスクトレイに常駐し、音声出力デバイスをワンクリックで切り替えるアプリ。

## 機能

- **左クリック**: ローテーション対象のデバイスを順番に切り替え
- **右クリック**: デバイス一覧メニューを表示し、ローテーション対象を選択

## 技術スタック

- [Tauri v2](https://v2.tauri.app/) - 軽量デスクトップアプリフレームワーク
- Rust - バックエンド (Windows Core Audio API)
- [com-policy-config](https://crates.io/crates/com-policy-config) - デフォルト音声デバイス切り替え

## 必要環境

- [mise](https://mise.jdx.dev/) - ツールバージョン管理

`mise.toml` で以下のツールを管理:

| ツール | バージョン |
|--------|-----------|
| Node.js | 24.13.0 |
| pnpm | 10.28.2 |
| Rust | stable |

## セットアップ

```bash
mise install
pnpm install
```

## 開発

```bash
pnpm tauri dev
```

## ビルド

```bash
pnpm tauri build
```

`src-tauri/target/release/bundle/` に `.msi` インストーラーと `.exe` が生成されます。

## プロジェクト構成

```
src/                    # フロントエンド (最小 - トレイ専用)
src-tauri/
  src/
    main.rs             # エントリポイント
    tray.rs             # トレイアイコン・メニュー・イベント処理
    audio.rs            # デバイス列挙・切り替え (Windows Core Audio API)
    config.rs           # 設定永続化 (ローテーション対象デバイス)
  Cargo.toml
  tauri.conf.json
```