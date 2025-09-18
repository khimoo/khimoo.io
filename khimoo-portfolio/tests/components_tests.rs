use khimoo_portfolio::home::components::{create_node_registry_from_articles, find_author_article};
use khimoo_portfolio::home::components::{ContainerBound, NodeContent, NodeId};
use khimoo_portfolio::home::data_loader::{ArticlesData, ProcessedArticle, ProcessedMetadata};

fn create_test_article(slug: &str, title: &str, author_image: Option<String>) -> ProcessedArticle {
    ProcessedArticle {
        slug: slug.to_string(),
        title: title.to_string(),
        content: "Test content".to_string(),
        metadata: ProcessedMetadata {
            title: title.to_string(),
            home_display: true,
            category: None,
            importance: Some(3),
            related_articles: vec![],
            tags: vec![],
            created_at: None,
            updated_at: None,
            author_image,
        },
        file_path: format!("{}.md", slug),
        outbound_links: vec![],
        inbound_count: 0,
        processed_at: "2024-01-01T00:00:00Z".to_string(),
    }
}

#[test]
fn test_find_author_article_none_found() {
    let articles_data = ArticlesData {
        articles: vec![
            create_test_article("article1", "Article 1", None),
            create_test_article("article2", "Article 2", None),
        ],
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        total_count: 2,
        home_articles: vec!["article1".to_string(), "article2".to_string()],
    };

    let result = find_author_article(&articles_data);
    assert!(result.is_none());
}

#[test]
fn test_find_author_article_single_found() {
    let articles_data = ArticlesData {
        articles: vec![
            create_test_article("article1", "Article 1", None),
            create_test_article("author", "About Me", Some("/images/profile.jpg".to_string())),
            create_test_article("article2", "Article 2", None),
        ],
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        total_count: 3,
        home_articles: vec!["article1".to_string(), "author".to_string(), "article2".to_string()],
    };

    let result = find_author_article(&articles_data);
    assert!(result.is_some());
    let author_article = result.unwrap();
    assert_eq!(author_article.slug, "author");
    assert_eq!(author_article.title, "About Me");
    assert_eq!(author_article.metadata.author_image, Some("/images/profile.jpg".to_string()));
}

#[test]
fn test_find_author_article_multiple_found() {
    let articles_data = ArticlesData {
        articles: vec![
            create_test_article("author1", "About Me 1", Some("/images/profile1.jpg".to_string())),
            create_test_article("article1", "Article 1", None),
            create_test_article("author2", "About Me 2", Some("/images/profile2.jpg".to_string())),
        ],
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        total_count: 3,
        home_articles: vec!["author1".to_string(), "article1".to_string(), "author2".to_string()],
    };

    let result = find_author_article(&articles_data);
    assert!(result.is_some());
    let author_article = result.unwrap();
    assert_eq!(author_article.slug, "author1");
    assert_eq!(author_article.title, "About Me 1");
    assert_eq!(author_article.metadata.author_image, Some("/images/profile1.jpg".to_string()));
}

#[test]
fn test_find_author_article_empty_articles() {
    let articles_data = ArticlesData {
        articles: vec![],
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        total_count: 0,
        home_articles: vec![],
    };

    let result = find_author_article(&articles_data);
    assert!(result.is_none());
}

#[test]
fn test_create_node_registry_with_author_metadata() {
    let articles_data = ArticlesData {
        articles: vec![
            create_test_article("author", "About Khimoo", Some("/images/profile.jpg".to_string())),
            create_test_article("article1", "Article 1", None),
        ],
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        total_count: 2,
        home_articles: vec!["author".to_string(), "article1".to_string()],
    };

    let container_bound = ContainerBound {
        width: 800.0,
        height: 600.0,
        ..Default::default()
    };

    let (registry, id_to_slug) = create_node_registry_from_articles(&articles_data, &container_bound);

    assert!(registry.positions.contains_key(&NodeId(0)));
    assert!(registry.contents.contains_key(&NodeId(0)));

    if let Some(NodeContent::Author { name, image_url, bio }) = registry.contents.get(&NodeId(0)) {
        assert_eq!(name, "About Khimoo");
        assert_eq!(image_url, "/images/profile.jpg");
        assert_eq!(bio, &None);
    } else {
        panic!("Author node should have NodeContent::Author content");
    }

    assert_eq!(id_to_slug.get(&NodeId(0)), Some(&"author".to_string()));
    assert_eq!(registry.get_node_importance(NodeId(0)), Some(5));
    assert_eq!(registry.get_node_inbound_count(NodeId(0)), 0);
}

#[test]
fn test_create_node_registry_fallback_without_author_metadata() {
    let articles_data = ArticlesData {
        articles: vec![
            create_test_article("article1", "Article 1", None),
            create_test_article("article2", "Article 2", None),
        ],
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        total_count: 2,
        home_articles: vec!["article1".to_string(), "article2".to_string()],
    };

    let container_bound = ContainerBound {
        width: 800.0,
        height: 600.0,
        ..Default::default()
    };

    let (registry, _id_to_slug) = create_node_registry_from_articles(&articles_data, &container_bound);

    assert!(registry.positions.contains_key(&NodeId(0)));
    assert!(registry.contents.contains_key(&NodeId(0)));

    if let Some(NodeContent::Text(text)) = registry.contents.get(&NodeId(0)) {
        assert_eq!(text, "Author");
    } else {
        panic!("Author node should have fallback text content");
    }
} 