use std::collections::HashMap;

use khimoo_portfolio::article_processing::{
    ArticleMetadata,
    ExtractedLink,
    FrontMatterParser,
    LinkExtractor,
    LinkType,
};
use khimoo_portfolio::article_processing::{
    LinkValidator,
    ProcessedArticleRef,
    ValidationError,
    ValidationErrorType,
    ValidationReport,
    ValidationReportFormatter,
    ValidationSummary,
};

#[test]
fn test_parse_complete_front_matter() {
    let content = r#"---

title: "Test Article"

home_display: true

category: "programming"

importance: 4

related_articles: ["article1", "article2"]

tags: ["rust", "test"]

created_at: "2024-01-01T00:00:00Z"

updated_at: "2024-01-02T00:00:00Z"

---



# Test Content



This is the markdown content.

"#;

    let (metadata, markdown) = FrontMatterParser::parse(content).unwrap();
    
    assert_eq!(metadata.title, "Test Article");
    assert_eq!(metadata.home_display, true);
    assert_eq!(metadata.category, Some("programming".to_string()));
    assert_eq!(metadata.importance, 4);
    assert_eq!(metadata.related_articles, vec!["article1", "article2"]);
    assert_eq!(metadata.tags, vec!["rust", "test"]);
    assert_eq!(metadata.created_at, Some("2024-01-01T00:00:00Z".to_string()));
    assert_eq!(metadata.updated_at, Some("2024-01-02T00:00:00Z".to_string()));
    
    assert!(markdown.trim().starts_with("# Test Content"));
}

#[test]
fn test_parse_minimal_front_matter() {
    let content = r#"---

title: "Minimal Article"

---



# Minimal Content

"#;

    let (metadata, markdown) = FrontMatterParser::parse(content).unwrap();
    
    assert_eq!(metadata.title, "Minimal Article");
    assert_eq!(metadata.home_display, false); // default
    assert_eq!(metadata.category, None);
    assert_eq!(metadata.importance, 3); // default
    assert!(metadata.related_articles.is_empty());
    assert!(metadata.tags.is_empty());
    
    assert!(markdown.trim().starts_with("# Minimal Content"));
}

#[test]
fn test_parse_no_front_matter() {
    let content = "# Just Markdown\n\nNo front matter here.";
    
    let (metadata, markdown) = FrontMatterParser::parse(content).unwrap();
    
    assert_eq!(metadata.title, "Untitled"); // default
    assert_eq!(metadata.home_display, false);
    assert_eq!(metadata.importance, 3);
    
    assert_eq!(markdown, content);
}

#[test]
fn test_parse_tags_from_front_matter() {
    let content = r#"---

title: "Tagged Article"

tags: ["rust", "programming", "web-development"]

---



# Tagged Content



This article has tags in front matter only.

"#;

    let (metadata, _markdown) = FrontMatterParser::parse(content).unwrap();
    
    assert_eq!(metadata.title, "Tagged Article");
    assert_eq!(metadata.tags, vec!["rust", "programming", "web-development"]);
}

#[test]
fn test_validate_metadata_valid() {
    let metadata = ArticleMetadata {
        title: "Valid Article".to_string(),
        home_display: true,
        category: Some("test".to_string()),
        importance: 3,
        related_articles: vec!["article1".to_string()],
        tags: vec!["tag1".to_string()],
        created_at: Some("2024-01-01T00:00:00Z".to_string()),
        updated_at: Some("2024-01-02T00:00:00Z".to_string()),
        author_image: None,
    };

    assert!(FrontMatterParser::validate_metadata(&metadata).is_ok());
}

#[test]
fn test_validate_metadata_invalid_importance() {
    let metadata = ArticleMetadata {
        title: "Test".to_string(),
        importance: 6, // Invalid: > 5
        ..Default::default()
    };

    let result = FrontMatterParser::validate_metadata(&metadata);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Importance must be between 1 and 5"));
}

#[test]
fn test_validate_metadata_empty_title() {
    let metadata = ArticleMetadata {
        title: "   ".to_string(), // Empty after trim
        ..Default::default()
    };

    let result = FrontMatterParser::validate_metadata(&metadata);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Title cannot be empty"));
}

#[test]
fn test_validate_metadata_invalid_datetime() {
    let metadata = ArticleMetadata {
        title: "Test".to_string(),
        created_at: Some("invalid-datetime".to_string()),
        ..Default::default()
    };

    let result = FrontMatterParser::validate_metadata(&metadata);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid created_at datetime format"));
}

#[test]
fn test_extract_wiki_links() {
    let extractor = LinkExtractor::new().unwrap();
    let content = "This is a [[test article]] and another [[Second Article]].";
    
    let links = extractor.extract_links(content);
    
    assert_eq!(links.len(), 2);
    
    assert_eq!(links[0].target_slug, "test-article");
    assert_eq!(links[0].link_type, LinkType::WikiLink);
    assert_eq!(links[0].original_text, "[[test article]]");
    assert!(links[0].context.contains("test article"));
    
    assert_eq!(links[1].target_slug, "second-article");
    assert_eq!(links[1].link_type, LinkType::WikiLink);
    assert_eq!(links[1].original_text, "[[Second Article]]");
}

#[test]
fn test_extract_markdown_links() {
    let extractor = LinkExtractor::new().unwrap();
    let content = "Check out [this article](article-slug) and [another one](second-slug).";
    
    let links = extractor.extract_links(content);
    
    assert_eq!(links.len(), 2);
    
    assert_eq!(links[0].target_slug, "article-slug");
    assert_eq!(links[0].link_type, LinkType::MarkdownLink);
    assert_eq!(links[0].original_text, "[this article](article-slug)");
    
    assert_eq!(links[1].target_slug, "second-slug");
    assert_eq!(links[1].link_type, LinkType::MarkdownLink);
    assert_eq!(links[1].original_text, "[another one](second-slug)");
}

#[test]
fn test_extract_mixed_links() {
    let extractor = LinkExtractor::new().unwrap();
    let content = r#"
        Start with [[wiki link]] then [markdown link](slug-here).
        Another [[Wiki Article]] and [external link](https://example.com).
        "#;
    
    let links = extractor.extract_links(content);
    
    // Should extract 3 links (excluding external http link)
    assert_eq!(links.len(), 3);
    
    // Check they are in order of appearance
    assert_eq!(links[0].link_type, LinkType::WikiLink);
    assert_eq!(links[0].target_slug, "wiki-link");
    
    assert_eq!(links[1].link_type, LinkType::MarkdownLink);
    assert_eq!(links[1].target_slug, "slug-here");
    
    assert_eq!(links[2].link_type, LinkType::WikiLink);
    assert_eq!(links[2].target_slug, "wiki-article");
}

#[test]
fn test_ignore_external_links() {
    let extractor = LinkExtractor::new().unwrap();
    let content = r#"
        Internal: [article](internal-slug)
        External: [website](https://example.com)
        Email: [contact](mailto:test@example.com)
        "#;
    
    let links = extractor.extract_links(content);
    
    // Should only extract the internal link
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].target_slug, "internal-slug");
    assert_eq!(links[0].link_type, LinkType::MarkdownLink);
}

#[test]
fn test_slug_generation_from_title() {
    let extractor = LinkExtractor::new().unwrap();
    
    assert_eq!(extractor.generate_slug_from_title("Simple Title"), "simple-title");
    assert_eq!(extractor.generate_slug_from_title("Title With Numbers 123"), "title-with-numbers-123");
    assert_eq!(extractor.generate_slug_from_title("Special!@#$%Characters"), "specialcharacters");
    assert_eq!(extractor.generate_slug_from_title("  Trimmed  Spaces  "), "trimmed-spaces");
    assert_eq!(extractor.generate_slug_from_title("Multiple---Dashes"), "multiple-dashes");
}

#[test]
fn test_context_extraction() {
    let extractor = LinkExtractor::new().unwrap();
    let content = "This is a long sentence with a [[test link]] in the middle of it for context testing.";
    
    let links = extractor.extract_links(content);
    
    assert_eq!(links.len(), 1);
    let context = &links[0].context;
    
    // Context should include surrounding text
    assert!(context.contains("long sentence"));
    assert!(context.contains("test link"));
    assert!(context.contains("middle"));
}

#[test]
fn test_context_with_multiline() {
    let extractor = LinkExtractor::new().unwrap();
    let content = r#"
        This is the first line.
        
        This line contains a [[test link]] here.
        
        This is the last line.
        "#;
    
    let links = extractor.extract_links(content);
    
    assert_eq!(links.len(), 1);
    let context = &links[0].context;
    
    // Context should be cleaned up (no excessive whitespace)
    assert!(context.contains("test link"));
    assert!(!context.contains("\n\n")); // Should not have multiple newlines
}

#[test]
fn test_real_article_patterns() {
    let extractor = LinkExtractor::new().unwrap();
    
    // Test pattern from rust-async.md
    let content = r#"
        非同期プログラミングを理解するには、まず[[tokio-basics]]を理解することから始めましょう。
        実用的な[パターン集](async-patterns)も参考になります。
        
        [[hello]]の記事でも触れましたが、非同期処理は重要です。
        "#;
    
    let links = extractor.extract_links(content);
    
    assert_eq!(links.len(), 3);
    
    // Check specific patterns
    assert_eq!(links[0].target_slug, "tokio-basics");
    assert_eq!(links[0].link_type, LinkType::WikiLink);
    
    assert_eq!(links[1].target_slug, "async-patterns");
    assert_eq!(links[1].link_type, LinkType::MarkdownLink);
    
    assert_eq!(links[2].target_slug, "hello");
    assert_eq!(links[2].link_type, LinkType::WikiLink);
}

#[test]
fn test_broken_link_patterns() {
    let extractor = LinkExtractor::new().unwrap();
    
    // Test pattern from broken-link-test.md
    let content = r#"
        - [[存在しない記事]]へのwikiリンク
        - [壊れたリンク](broken-slug)へのmarkdownリンク
        "#;
    
    let links = extractor.extract_links(content);
    
    assert_eq!(links.len(), 2);
    
    assert_eq!(links[0].target_slug, "存在しない記事");
    assert_eq!(links[0].link_type, LinkType::WikiLink);
    
    assert_eq!(links[1].target_slug, "broken-slug");
    assert_eq!(links[1].link_type, LinkType::MarkdownLink);
}

#[test]
fn test_edge_cases() {
    let extractor = LinkExtractor::new().unwrap();
    
    // Empty content
    assert_eq!(extractor.extract_links("").len(), 0);
    
    // No links
    assert_eq!(extractor.extract_links("Just plain text with no links.").len(), 0);
    
    // Malformed links
    let malformed = "[[incomplete link] and [incomplete](";
    assert_eq!(extractor.extract_links(malformed).len(), 0);
    
    // Nested brackets (regex will match the first complete bracket pair)
    let nested = "[[outer [[inner]] link]]";
    let links = extractor.extract_links(nested);
    // The regex will match "[[outer [[inner]]" - the first [[ to the first ]]
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].target_slug, "outer-inner");
}

#[test]
fn test_link_validator_broken_links() {
    let articles = vec![
        ProcessedArticleRef {
            slug: "article1".to_string(),
            title: "Article 1".to_string(),
            metadata: ArticleMetadata::default(),
            outbound_links: vec![
                ExtractedLink {
                    target_slug: "article2".to_string(),
                    link_type: LinkType::WikiLink,
                    context: "Link to article2".to_string(),
                    position: 0,
                    original_text: "[[article2]]".to_string(),
                },
                ExtractedLink {
                    target_slug: "nonexistent".to_string(),
                    link_type: LinkType::WikiLink,
                    context: "Broken link".to_string(),
                    position: 20,
                    original_text: "[[nonexistent]]".to_string(),
                },
            ],
            file_path: "article1.md".to_string(),
        },
        ProcessedArticleRef {
            slug: "article2".to_string(),
            title: "Article 2".to_string(),
            metadata: ArticleMetadata::default(),
            outbound_links: vec![],
            file_path: "article2.md".to_string(),
        },
    ];

    let validator = LinkValidator::new(&articles);
    let report = validator.validate_all().unwrap();

    assert_eq!(report.summary.total_articles, 2);
    assert_eq!(report.summary.broken_links, 1);
    assert_eq!(report.errors.len(), 1);
    
    let error = &report.errors[0];
    assert_eq!(error.error_type, ValidationErrorType::BrokenLink);
    assert_eq!(error.source_article, "article1");
    assert_eq!(error.target_reference, "nonexistent");
}

#[test]
fn test_link_validator_invalid_related_articles() {
    let mut metadata = ArticleMetadata::default();
    metadata.related_articles = vec!["existing".to_string(), "missing".to_string()];

    let articles = vec![
        ProcessedArticleRef {
            slug: "main".to_string(),
            title: "Main Article".to_string(),
            metadata,
            outbound_links: vec![],
            file_path: "main.md".to_string(),
        },
        ProcessedArticleRef {
            slug: "existing".to_string(),
            title: "Existing Article".to_string(),
            metadata: ArticleMetadata::default(),
            outbound_links: vec![],
            file_path: "existing.md".to_string(),
        },
    ];

    let validator = LinkValidator::new(&articles);
    let report = validator.validate_all().unwrap();

    assert_eq!(report.summary.invalid_references, 1);
    assert_eq!(report.errors.len(), 1);
    
    let error = &report.errors[0];
    assert_eq!(error.error_type, ValidationErrorType::InvalidRelatedArticle);
    assert_eq!(error.source_article, "main");
    assert_eq!(error.target_reference, "missing");
}

#[test]
fn test_link_validator_warnings() {
    let mut high_importance_metadata = ArticleMetadata::default();
    high_importance_metadata.importance = 5;

    let mut low_importance_metadata = ArticleMetadata::default();
    low_importance_metadata.importance = 1;

    let articles = vec![
        ProcessedArticleRef {
            slug: "high-importance".to_string(),
            title: "High Importance".to_string(),
            metadata: high_importance_metadata,
            outbound_links: vec![],
            file_path: "high.md".to_string(),
        },
        ProcessedArticleRef {
            slug: "low-importance".to_string(),
            title: "Low Importance".to_string(),
            metadata: low_importance_metadata,
            outbound_links: vec![
                ExtractedLink {
                    target_slug: "high-importance".to_string(),
                    link_type: LinkType::WikiLink,
                    context: "Link to high importance".to_string(),
                    position: 0,
                    original_text: "[[high-importance]]".to_string(),
                },
            ],
            file_path: "low.md".to_string(),
        },
        ProcessedArticleRef {
            slug: "orphaned".to_string(),
            title: "Orphaned Article".to_string(),
            metadata: ArticleMetadata::default(),
            outbound_links: vec![],
            file_path: "orphaned.md".to_string(),
        },
    ];

    let validator = LinkValidator::new(&articles);
    let report = validator.validate_all().unwrap();

    // Should have warnings for high importance with few links and orphaned article
    assert!(report.warnings.len() >= 2);
    
    let warning_types: Vec<_> = report.warnings.iter().map(|w| &w.warning_type).collect();
    assert!(warning_types.contains(&&khimoo_portfolio::article_processing::ValidationWarningType::HighImportanceWithFewLinks));
    assert!(warning_types.contains(&&khimoo_portfolio::article_processing::ValidationWarningType::MissingBacklinks));
}

#[test]
fn test_similarity_calculation() {
    let validator = LinkValidator::new(&[]);
    
    // Exact match
    assert_eq!(validator.calculate_similarity("test", "test"), 1.0);
    
    // Similar strings
    let similarity = validator.calculate_similarity("article-name", "article-names");
    assert!(similarity > 0.8);
    
    // Different strings
    let similarity = validator.calculate_similarity("completely", "different");
    assert!(similarity < 0.3);
}

#[test]
fn test_suggestion_generation() {
    let articles = vec![
        ProcessedArticleRef {
            slug: "rust-async".to_string(),
            title: "Rust Async".to_string(),
            metadata: ArticleMetadata::default(),
            outbound_links: vec![],
            file_path: "rust-async.md".to_string(),
        },
        ProcessedArticleRef {
            slug: "tokio-basics".to_string(),
            title: "Tokio Basics".to_string(),
            metadata: ArticleMetadata::default(),
            outbound_links: vec![],
            file_path: "tokio-basics.md".to_string(),
        },
    ];

    let validator = LinkValidator::new(&articles);
    
    // Should suggest similar article
    let suggestion = validator.suggest_similar_article("rust-asyncs");
    assert!(suggestion.is_some());
    assert!(suggestion.unwrap().contains("rust-async"));
    
    // Should not suggest for very different strings
    let suggestion = validator.suggest_similar_article("completely-different");
    assert!(suggestion.is_none());
}

#[test]
fn test_validation_report_json_format() {
    let report = ValidationReport {
        validation_date: "2024-01-01T00:00:00Z".to_string(),
        summary: ValidationSummary {
            total_articles: 2,
            total_links: 1,
            broken_links: 1,
            invalid_references: 0,
            orphaned_articles: 0,
            circular_references: 0,
            articles_with_errors: 1,
            articles_with_warnings: 0,
        },
        errors: vec![
            ValidationError {
                error_type: ValidationErrorType::BrokenLink,
                source_article: "test".to_string(),
                target_reference: "missing".to_string(),
                context: Some("test context".to_string()),
                line_number: None,
                suggestion: Some("Did you mean 'existing'?".to_string()),
            }
        ],
        warnings: vec![],
        article_stats: HashMap::new(),
    };

    let json = ValidationReportFormatter::format_json(&report).unwrap();
    assert!(json.contains("validation_date"));
    assert!(json.contains("BrokenLink"));
    assert!(json.contains("test context"));
    
    // Verify it's valid JSON by parsing it back
    let parsed: ValidationReport = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.summary.broken_links, 1);
}

#[test]
fn test_validation_report_console_format() {
    let report = ValidationReport {
        validation_date: "2024-01-01T00:00:00Z".to_string(),
        summary: ValidationSummary {
            total_articles: 2,
            total_links: 1,
            broken_links: 1,
            invalid_references: 0,
            orphaned_articles: 0,
            circular_references: 0,
            articles_with_errors: 1,
            articles_with_warnings: 0,
        },
        errors: vec![
            ValidationError {
                error_type: ValidationErrorType::BrokenLink,
                source_article: "test".to_string(),
                target_reference: "missing".to_string(),
                context: Some("test context".to_string()),
                line_number: None,
                suggestion: Some("Did you mean 'existing'?".to_string()),
            }
        ],
        warnings: vec![],
        article_stats: HashMap::new(),
    };

    let console = ValidationReportFormatter::format_console(&report);
    assert!(console.contains("🔍 Link Validation Report"));
    assert!(console.contains("📊 Summary:"));
    assert!(console.contains("❌ Broken links: 1"));
    assert!(console.contains("🔗 Broken Link: test → missing"));
    assert!(console.contains("💡 Did you mean 'existing'?"));
}

#[test]
fn test_validation_report_ci_summary() {
    // Test successful validation
    let success_report = ValidationReport {
        validation_date: "2024-01-01T00:00:00Z".to_string(),
        summary: ValidationSummary {
            total_articles: 2,
            total_links: 1,
            broken_links: 0,
            invalid_references: 0,
            orphaned_articles: 0,
            circular_references: 0,
            articles_with_errors: 0,
            articles_with_warnings: 1,
        },
        errors: vec![],
        warnings: vec![],
        article_stats: HashMap::new(),
    };

    let summary = ValidationReportFormatter::format_ci_summary(&success_report);
    assert!(summary.contains("✅ All links valid"));
    assert!(summary.contains("(1 warnings)"));

    // Test failed validation
    let failed_report = ValidationReport {
        validation_date: "2024-01-01T00:00:00Z".to_string(),
        summary: ValidationSummary {
            total_articles: 2,
            total_links: 1,
            broken_links: 2,
            invalid_references: 1,
            orphaned_articles: 0,
            circular_references: 0,
            articles_with_errors: 1,
            articles_with_warnings: 0,
        },
        errors: vec![],
        warnings: vec![],
        article_stats: HashMap::new(),
    };

    let summary = ValidationReportFormatter::format_ci_summary(&failed_report);
    assert!(summary.contains("❌ Validation failed:"));
    assert!(summary.contains("2 broken links"));
    assert!(summary.contains("1 invalid references"));
} 