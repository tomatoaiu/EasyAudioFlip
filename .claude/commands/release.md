---
name: release
description: バージョンを指定してリリースを実行する
argument-hint: [version]
disable-model-invocation: true
---

リリースを実行する。

## 手順

1. `git status` で clean かつ `main` ブランチであることを確認。そうでなければ中止
2. `$ARGUMENTS` が `MAJOR.MINOR.PATCH` 形式であることを確認。不正なら中止
3. `node scripts/release.mjs $ARGUMENTS` を実行
4. リリースされたバージョンを報告
