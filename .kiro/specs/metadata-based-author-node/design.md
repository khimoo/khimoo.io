# Design Document

## Overview

この設計では、現在ハードコーディングされている作者ノード（ID=0）を、メタデータベースのアプローチに変更します。`author_image`フィールドを持つmarkdownファイルを作者ノードとして認識し、同時にそのフィールドで指定された画像を表示します。既存のコードへの影響を最小限に抑えながら、より柔軟で保守性の高いシステムを実現します。

## Architecture

### Current System Analysis

現在のシステムでは以下の構造になっています：

1. **components.rs**: `create_node_registry_from_articles`関数でID=0の作者ノードをハードコーディング
2. **types.rs**: `NodeContent::Author`型が既に定義済み、画像表示機能も実装済み
3. **article_processing.rs**: `ArticleMetadata`構造体でメタデータを管理
4. **data_loader.rs**: `ProcessedMetadata`構造体で処理済みメタデータを管理

### New Architecture

新しいアーキテクチャでは以下の変更を行います：

```
┌─────────────────────────────────────────────────────────────┐
│                    Metadata Processing                       │
├─────────────────────────────────────────────────────────────┤
│ 1. ArticleMetadata に author_image フィールドを追加         │
│ 2. ProcessedMetadata に author_image フィールドを追加       │
│ 3. author_image の存在で作者ノードを識別                   │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                   Unified Node Processing                    │
├─────────────────────────────────────────────────────────────┤
│ 1. 全記事（作者記事含む）を統一的に処理                    │
│ 2. author_image を持つ記事は NodeContent::Author を作成    │
│ 3. 一般記事は NodeContent::Text を作成                     │
│ 4. 全記事に対して通常のID割り当てロジックを適用            │
│ 5. リンク処理も全記事に対して統一的に実行                  │
└─────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────┐
│                    Node Rendering                           │
├─────────────────────────────────────────────────────────────┤
│ NodeContent::Author の render_content() で画像を表示       │
│ NodeContent::Text の render_content() でテキスト表示       │
│ （既存の実装をそのまま使用）                               │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. Metadata Structure Extensions

#### ArticleMetadata (article_processing.rs)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleMetadata {
    // 既存フィールド...
    pub author_image: Option<String>, // 新規追加
}
```

#### ProcessedMetadata (data_loader.rs)
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessedMetadata {
    // 既存フィールド...
    pub author_image: Option<String>, // 新規追加
}
```

### 2. Unified Article Processing Logic

作者ノード検索ロジックを削除し、統一的な記事処理に変更：

```rust
// 作者ノード検索関数は削除
// 代わりに記事処理時に author_image の存在をチェック

fn determine_node_content(article: &ProcessedArticle) -> NodeContent {
    if let Some(image_url) = &article.metadata.author_image {
        NodeContent::Author {
            name: article.title.clone(),
            image_url: image_url.clone(),
            bio: None,
        }
    } else {
        NodeContent::Text(article.title.clone())
    }
}
```

### 3. Unified Node Registry Creation

`create_node_registry_from_articles`関数を完全に書き直し：

```rust
fn create_node_registry_from_articles(
    articles_data: &ArticlesData, 
    container_bound: &ContainerBound
) -> (NodeRegistry, HashMap<NodeId, String>) {
    let mut reg = NodeRegistry::new();
    let mut slug_to_id = HashMap::new();
    let mut id_to_slug = HashMap::new();
    let mut next_id = 1u32;
    
    let center_x = container_bound.width / 2.0;
    let center_y = container_bound.height / 2.0;
    
    // home_display=trueの記事のみをノードとして追加（作者記事も含む）
    let home_articles: Vec<_> = articles_data.articles.iter()
        .filter(|article| article.metadata.home_display)
        .collect();

    if home_articles.is_empty() {
        // フォールバック：従来のハードコーディング方式
        reg.add_node(
            NodeId(next_id),
            Position { x: center_x, y: center_y },
            40,
            NodeContent::Text("Author".to_string()),
        );
        slug_to_id.insert("author".to_string(), NodeId(next_id));
        id_to_slug.insert(NodeId(next_id), "author".to_string());
        return (reg, id_to_slug);
    }

    // 作者記事を特定（最初に見つかったもの）
    let author_article = home_articles.iter()
        .find(|article| article.metadata.author_image.is_some());

    // 円形配置の計算
    let radius = (container_bound.width.min(container_bound.height) * 0.3).max(150.0);
    let angle_step = 2.0 * std::f32::consts::PI / home_articles.len() as f32;

    for (index, article) in home_articles.iter().enumerate() {
        let angle = index as f32 * angle_step;
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();

        let node_id = NodeId(next_id);
        
        // 作者記事かどうかで表示内容を決定
        let content = determine_node_content(article);
        
        // 作者記事の場合は中央に配置し、大きめのサイズにする
        let (position, base_radius) = if article.metadata.author_image.is_some() {
            (Position { x: center_x, y: center_y }, 60)
        } else {
            (Position { x, y }, 30)
        };

        reg.add_node(node_id, position, base_radius, content);

        // 重要度とリンク数を設定
        reg.set_node_importance(node_id, article.metadata.importance.unwrap_or(3));
        reg.set_node_inbound_count(node_id, article.inbound_count);

        slug_to_id.insert(article.slug.clone(), node_id);
        id_to_slug.insert(node_id, article.slug.clone());
        next_id += 1;
    }

    // 記事間のリンクを追加（作者記事も含む）
    for article in &home_articles {
        if let Some(&from_id) = slug_to_id.get(&article.slug) {
            for link in &article.outbound_links {
                if let Some(&to_id) = slug_to_id.get(&link.target_slug) {
                    reg.add_edge(from_id, to_id);
                }
            }
        }
    }

    (reg, id_to_slug)
}
```

## Data Models

### Author Metadata Schema

作者ノードのmarkdownファイルは以下のメタデータ構造を持ちます：

```yaml
---
title: "Your Name"
author_image: "/path/to/profile.jpg"
home_display: true   # 作者ノードもhome画面に作者ノードとして表示されるためここはtrueになる(ただし、この値は扱わないだろう)
importance: 5        # 最高重要度
category: "author"   # オプション
tags: ["about", "profile"]
---

# About Me

ここには自己紹介およびブログの概要を記載する
```

### Image Path Resolution

画像パスは以下の形式をサポート：

1. **相対パス**: `"images/profile.jpg"` → `/khimoo.io/images/profile.jpg`
2. **絶対パス**: `"/assets/profile.jpg"` → `/assets/profile.jpg`
3. **外部URL**: `"https://example.com/profile.jpg"` → そのまま使用

## Error Handling

### Graceful Degradation Strategy

1. **author_image フィールドが存在しない場合**
   - 従来のハードコーディング方式にフォールバック
   - ログに警告を出力

2. **複数の author_image フィールドが存在する場合**
   - 最初に見つかったファイルを使用
   - 他のファイルについて警告をログに出力

3. **画像ファイルが存在しない場合**
   - NodeContent::Author の render_content() で自動的にフォールバック
   - 画像が読み込めない場合はテキスト表示

4. **メタデータ解析エラー**
   - エラーをログに出力
   - フォールバック方式を使用

### Error Logging Strategy

```rust
// 作者ノード検索時のログ出力例
if let Some(author_article) = find_author_article(articles_data) {
    web_sys::console::log_1(&format!(
        "Found author node: {} with image: {}", 
        author_article.title,
        author_article.metadata.author_image.as_ref().unwrap()
    ).into());
} else {
    web_sys::console::warn_1(&"No author_image found, using fallback author node".into());
}
```

## Testing Strategy

### Unit Tests

1. **メタデータ解析テスト**
   - `author_image` フィールドの正しい解析
   - 存在しない場合のデフォルト値

2. **作者ノード検索テスト**
   - 単一の作者ノードが正しく検索される
   - 複数の作者ノードがある場合の動作
   - 作者ノードが存在しない場合のフォールバック

3. **ノード作成テスト**
   - `NodeContent::Author` の正しい作成
   - 画像URLの正しい設定
   - フォールバック時の `NodeContent::Text` 作成

### Integration Tests

1. **エンドツーエンドテスト**
   - 作者ノードが正しく表示される
   - 画像が正しく読み込まれる
   - クリック動作が正常に機能する

2. **データローディングテスト**
   - `articles.json` から `author_image` が正しく読み込まれる
   - 処理済みデータに `author_image` が含まれる

### Test Data

テスト用の作者記事ファイル例：

```markdown
---
title: "Khimoo"
author_image: "/images/profile.jpg"
home_display: false
importance: 5
category: "author"
tags: ["about", "profile"]
---

# About Khimoo

Software developer passionate about Rust and web technologies.
```

## Performance Considerations

### Minimal Impact Design

1. **検索処理の最適化**
   - 作者ノード検索は初期化時に1回のみ実行
   - 線形検索だが記事数は限定的なので問題なし

2. **メモリ使用量**
   - 新しいフィールドは `Option<String>` なので最小限の追加
   - 既存のデータ構造をほぼそのまま使用

3. **レンダリング性能**
   - 既存の `NodeContent::Author` の実装を使用
   - 追加のレンダリングオーバーヘッドなし

### Caching Strategy

- 作者ノードの情報は初期化時にキャッシュ
- 画像の読み込みはブラウザの標準キャッシュに依存
- 必要に応じて将来的にService Workerでのキャッシュを検討

## Migration Strategy

### Phase 1: Metadata Structure Update
1. `ArticleMetadata` に `author_image` フィールドを追加
2. `ProcessedMetadata` に `author_image` フィールドを追加
3. 記事処理パイプラインでの新フィールドの処理

### Phase 2: Node Creation Logic Update
1. `find_author_article` 関数の実装
2. `create_node_registry_from_articles` 関数の更新
3. フォールバック機能の実装

### Phase 3: Testing and Validation
1. 単体テストの実装
2. 統合テストの実行
3. 実際の作者記事ファイルの作成

### Phase 4: Documentation and Cleanup
1. 使用方法のドキュメント作成
2. 既存のハードコーディング部分のコメント更新
3. エラーハンドリングの改善

## Security Considerations

### Image Source Validation

1. **パス検証**
   - 相対パスの場合は適切なベースURLを使用
   - ディレクトリトラバーサル攻撃の防止

2. **外部URL制限**
   - HTTPS URLのみ許可（本番環境）
   - 信頼できるドメインのホワイトリスト（将来的）

3. **画像サイズ制限**
   - CSSで最大サイズを制限
   - 大きすぎる画像による性能問題の防止

### Content Security Policy

- 画像の読み込み元に対するCSPの設定
- インライン画像（data: URL）の制限検討