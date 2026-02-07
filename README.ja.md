# EasyAudioFlip

Windows 11 のタスクトレイに常駐し、音声出力デバイスをワンクリックで切り替えるアプリ。

![demo](assets/demo.gif)

## 機能

- **左クリック**: ローテーション対象のデバイスを順番に切り替え
- **右クリック**: デバイス一覧パネルを表示し、ローテーション対象のオン/オフを切り替え

## 必要環境

- [mise](https://mise.jdx.dev/) - ツールバージョン管理

## セットアップ

```bash
mise install
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
