---
title: "Tokio入門"
home_display: true
category: "programming"
importance: 3
tags: ["rust", "tokio", "async"]
created_at: "2024-01-03T00:00:00Z"
updated_at: "2024-01-03T00:00:00Z"
---

# Tokio入門

Tokioは[[rust-async]]の基礎となる非同期ランタイムです。

## Tokioの特徴

- 高性能な非同期I/O
- タスクスケジューリング
- タイマーとタイムアウト

## 基本的な使用方法

```rust
#[tokio::main]
async fn main() {
    println!("Hello, Tokio!");
}
```

この記事は[[rust-async]]と密接に関連しており、非同期プログラミングの実装面を扱います。