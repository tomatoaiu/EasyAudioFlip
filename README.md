# EasyAudioFlip

Windows 11 のタスクトレイに常駐し、音声出力デバイスをワンクリックで切り替えるアプリ。

## 機能

- **左クリック**: ローテーション対象のデバイスを順番に切り替え
- **右クリック**: デバイス一覧パネルを表示し、ローテーション対象のオン/オフを切り替え

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
| Rust | 1.93.0 |

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

## リリース

```bash
node scripts/release.mjs <version>
```

バージョンバンプ、コミット、タグ作成、プッシュを一括実行します。プッシュ後に GitHub Actions が Windows 向けインストーラーをビルドしてリリースを作成します。