---
name: release
description: Release Please の pull request を確認して公開する
disable-model-invocation: true
---

Release Please で公開する。バージョンは手で指定しない。

## 手順

1. `main` が clean で、未コミットの変更がないことを確認する。そうでなければ中止
2. `github-actions[bot]` の open な Release Please pull request を確認する。なければ、`Prepare release` workflow の完了を待ってから再確認する
3. pull request の版が `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.lock`、`.release-please-manifest.json` で一致することを確認する
4. その pull request を squash merge する
5. `Release` workflow が成功し、対応する GitHub Release ができたことを確認して報告する
