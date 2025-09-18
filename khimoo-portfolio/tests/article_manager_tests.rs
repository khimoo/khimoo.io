use khimoo_portfolio::home::article_manager::ArticleManager;
use khimoo_portfolio::home::data_loader::{ArticlesData, ProcessedArticle, ProcessedMetadata, LinkGraphData};
use std::collections::HashMap;

fn create_test_article(slug: &str, title: &str, home_display: bool, importance: u8) -> ProcessedArticle {
    ProcessedArticle {
        slug: slug.to_string(),
        title: title.to_string(),
        content: format!("Content for {}", title),
        metadata: ProcessedMetadata {
            title: title.to_string(),
            home_display,
            category: Some("test".to_string()),
            importance: Some(importance),
            related_articles: Vec::new(),
            tags: vec!["test".to_string()],
            created_at: None,
            updated_at: None,
            author_image: None,
        },
        file_path: format!("articles/{}.md", slug),
        outbound_links: Vec::new(),
        inbound_count: 0,
        processed_at: "2024-01-01T00:00:00Z".to_string(),
    }
}

#[test]
fn test_article_manager_creation() {
    let manager = ArticleManager::new();
    assert_eq!(manager.articles.len(), 0);
    assert_eq!(manager.home_articles.len(), 0);
}

#[test]
fn test_load_from_data() {
    let mut manager = ArticleManager::new();

    let articles_data = ArticlesData {
        articles: vec![
            create_test_article("test1", "Test Article 1", true, 4),
            create_test_article("test2", "Test Article 2", false, 2),
        ],
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        total_count: 2,
        home_articles: vec!["test1".to_string()],
    };

    let link_graph_data = LinkGraphData {
        graph: HashMap::new(),
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        total_connections: 0,
        bidirectional_pairs: Some(0),
        direct_links: Some(0),
    };

    manager.load_from_data(articles_data, link_graph_data);

    assert_eq!(manager.lightweight_articles.len(), 2);
    assert_eq!(manager.home_articles.len(), 1);
    assert_eq!(manager.home_articles[0], "test1");
}

#[test]
fn test_get_home_articles() {
    let mut manager = ArticleManager::new();

    let articles_data = ArticlesData {
        articles: vec![
            create_test_article("test1", "Test Article 1", true, 4),
            create_test_article("test2", "Test Article 2", false, 2),
        ],
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        total_count: 2,
        home_articles: vec!["test1".to_string()],
    };

    let link_graph_data = LinkGraphData {
        graph: HashMap::new(),
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        total_connections: 0,
        bidirectional_pairs: Some(0),
        direct_links: Some(0),
    };

    manager.load_from_data(articles_data, link_graph_data);

    let home_articles = manager.get_home_articles();
    assert_eq!(home_articles.len(), 1);
    assert_eq!(home_articles[0].slug, "test1");
} 