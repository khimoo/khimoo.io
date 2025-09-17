# Test Articles Documentation

このディレクトリには、インタラクティブマインドマップポートフォリオの開発・テスト用記事が含まれています。

## 記事構成

### Phase 1: 基本記事セット（6記事）

1. **hello.md** - 導入記事（home_display: true, importance: 4）
2. **rust-async.md** - Rust非同期プログラミング（home_display: true, importance: 4）
3. **tokio-basics.md** - Tokio入門（home_display: true, importance: 3）
4. **async-patterns.md** - 非同期パターン集（home_display: false, importance: 2）
5. **web-development.md** - Web開発基礎（home_display: true, importance: 3）
6. **broken-link-test.md** - リンク切れテスト（home_display: false, importance: 1）

## リンク関係

### 相互リンク
- rust-async ↔ tokio-basics
- hello ↔ rust-async
- hello ↔ web-development

### 一方向リンク
- async-patterns → rust-async
- async-patterns → tokio-basics

### 意図的なリンク切れ（テスト用）
- broken-link-test → 存在しない記事

## カテゴリ分類

- **introduction**: hello
- **programming**: rust-async, tokio-basics, async-patterns
- **web**: web-development
- **test**: broken-link-test

## 重要度レベル

- **4**: hello, rust-async（最重要、大きなノード）
- **3**: tokio-basics, web-development（重要、中サイズノード）
- **2**: async-patterns（普通、小サイズノード）
- **1**: broken-link-test（最小、テスト用）

## ホーム画面表示

home_display=trueの記事（4記事）がホーム画面のノードとして表示されます：
- hello
- rust-async
- tokio-basics
- web-development

## タグ

記事内の#タグは抽出・記録されますが、関連性計算には使用されません。
デバッグ情報として活用されます。