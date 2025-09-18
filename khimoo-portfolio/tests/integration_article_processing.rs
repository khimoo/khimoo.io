#![cfg(not(target_arch = "wasm32"))]

use khimoo_portfolio::FrontMatterParser;
use khimoo_portfolio::home::data_loader::{ArticlesData, ProcessedArticle, ProcessedMetadata};

#[test]
fn test_front_matter_tag_integration() {
    let content = r#"---
:title: "Rustでの非同期プログラミング"
:home_display: true
:category: "programming"
:importance: 4
:related_articles: ["tokio-basics", "async-patterns"]
:tags: ["rust", "async", "programming"]
:created_at: "2024-01-02T00:00:00Z"
:updated_at: "2024-01-02T00:00:00Z"
:---
:
:# Rustでの非同期プログラミング
:
:Rustにおける非同期プログラミングの基礎について説明します。
:
:## 基本概念
:
:非同期プログラミングを理解するには、まず[[tokio-basics]]を理解することから始めましょう。
:実用的な[パターン集](async-patterns)も参考になります。
:
:## 主要な特徴
:
:- Future trait
:- async/await構文
:- 非同期ランタイム
:
:[[hello]]の記事でも触れましたが、非同期処理は現代のWebアプリケーション開発において重要な概念です。
:"#;

    let content = content.replace("\n:", "\n");

    let (metadata, _markdown_content) = FrontMatterParser::parse(&content).unwrap();
    assert_eq!(metadata.tags, vec!["rust", "async", "programming"]);
    assert_eq!(metadata.title, "Rustでの非同期プログラミング");
    assert_eq!(metadata.home_display, true);
    assert_eq!(metadata.category, Some("programming".to_string()));
    assert_eq!(metadata.importance, 4);
    assert_eq!(metadata.related_articles, vec!["tokio-basics", "async-patterns"]);
}

#[test]
fn test_front_matter_only_tags() {
    let content = r#"---
:title: "Test Article"
:tags: ["rust", "web", "programming"]
:---
:
:# Test Article
:
:This article covers various topics but tags are only managed in front matter.
:Content may contain words like rust and programming but they are not extracted as tags.
:"#;

    let content = content.replace("\n:", "\n");

    let (metadata, _markdown_content) = FrontMatterParser::parse(&content).unwrap();
    assert_eq!(metadata.tags, vec!["rust", "web", "programming"]);
    assert_eq!(metadata.title, "Test Article");
}

#[test]
fn test_node_navigation_integration() {
    let test_article = ProcessedArticle {
        slug: "test-article".to_string(),
        title: "Test Article".to_string(),
        content: "# Test Article\n\nThis is a test article.".to_string(),
        metadata: ProcessedMetadata {
            title: "Test Article".to_string(),
            home_display: true,
            category: Some("test".to_string()),
            importance: Some(3),
            related_articles: vec![],
            tags: vec!["test".to_string()],
            created_at: None,
            updated_at: None,
            author_image: None,
        },
        file_path: "articles/test-article.md".to_string(),
        outbound_links: vec![],
        inbound_count: 0,
        processed_at: "2024-01-01T00:00:00Z".to_string(),
    };

    let articles_data = ArticlesData {
        articles: vec![test_article],
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        total_count: 1,
        home_articles: vec!["test-article".to_string()],
    };

    assert_eq!(articles_data.articles.len(), 1);
    assert_eq!(articles_data.articles[0].slug, "test-article");
    assert_eq!(articles_data.articles[0].metadata.home_display, true);
    assert!(articles_data.home_articles.contains(&"test-article".to_string()));
} 