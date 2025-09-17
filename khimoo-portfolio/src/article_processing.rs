use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use chrono::DateTime;
use regex::Regex;
use yaml_front_matter::{Document, YamlFrontMatter};

/// Article metadata structure with default values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleMetadata {
    pub title: String,
    #[serde(default)]
    pub home_display: bool,
    pub category: Option<String>,
    #[serde(default = "default_importance")]
    pub importance: u8,
    #[serde(default)]
    pub related_articles: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Default for ArticleMetadata {
    fn default() -> Self {
        Self {
            title: "Untitled".to_string(),
            home_display: false,
            category: None,
            importance: default_importance(),
            related_articles: Vec::new(),
            tags: Vec::new(),
            created_at: None,
            updated_at: None,
        }
    }
}

fn default_importance() -> u8 {
    3
}

/// Types of links that can be extracted from markdown content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LinkType {
    WikiLink,      // [[article-name]] format
    MarkdownLink,  // [text](slug) format
}

/// Represents a link found in markdown content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedLink {
    pub target_slug: String,
    pub link_type: LinkType,
    pub context: String,
    pub position: usize,
    pub original_text: String,
}

/// Link extractor for markdown content
pub struct LinkExtractor {
    wiki_regex: Regex,
    markdown_regex: Regex,
}

impl LinkExtractor {
    /// Create a new link extractor with compiled regex patterns
    pub fn new() -> Result<Self> {
        let wiki_regex = Regex::new(r"\[\[([^\]]+)\]\]")
            .context("Failed to compile wiki link regex")?;
        let markdown_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")
            .context("Failed to compile markdown link regex")?;
        
        Ok(Self {
            wiki_regex,
            markdown_regex,
        })
    }

    /// Extract all links from markdown content
    pub fn extract_links(&self, content: &str) -> Vec<ExtractedLink> {
        let mut links = Vec::new();
        
        // Extract wiki-style links [[article-name]]
        for cap in self.wiki_regex.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            let target = cap.get(1).unwrap().as_str();
            let position = full_match.start();
            
            links.push(ExtractedLink {
                target_slug: self.generate_slug_from_title(target),
                link_type: LinkType::WikiLink,
                context: self.get_context(content, position, 100),
                position,
                original_text: full_match.as_str().to_string(),
            });
        }
        
        // Extract markdown-style links [text](slug)
        for cap in self.markdown_regex.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            let _text = cap.get(1).unwrap().as_str();
            let target = cap.get(2).unwrap().as_str();
            let position = full_match.start();
            
            // Only process internal links (not starting with http/https)
            if !target.starts_with("http") && !target.starts_with("mailto:") {
                links.push(ExtractedLink {
                    target_slug: target.to_string(),
                    link_type: LinkType::MarkdownLink,
                    context: self.get_context(content, position, 100),
                    position,
                    original_text: full_match.as_str().to_string(),
                });
            }
        }
        
        // Sort links by position for consistent ordering
        links.sort_by_key(|link| link.position);
        
        links
    }

    /// Generate a slug from article title (for wiki links)
    fn generate_slug_from_title(&self, title: &str) -> String {
        let slug = title
            .to_lowercase()
            .trim()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>()
            .trim_matches('-')
            .to_string();
        
        // Replace multiple consecutive dashes with single dash
        let re = Regex::new(r"-+").unwrap();
        re.replace_all(&slug, "-").to_string()
    }

    /// Get context around a link position
    fn get_context(&self, content: &str, position: usize, context_length: usize) -> String {
        // Work with character indices to handle Unicode properly
        let chars: Vec<char> = content.chars().collect();
        
        // Find the character position corresponding to the byte position
        let char_position = content[..position].chars().count();
        
        let half_length = context_length / 2;
        let start_char = char_position.saturating_sub(half_length);
        let end_char = std::cmp::min(char_position + half_length, chars.len());
        
        // Find word boundaries to avoid cutting words
        let start_boundary = self.find_char_word_boundary(&chars, start_char, true);
        let end_boundary = self.find_char_word_boundary(&chars, end_char, false);
        
        let context: String = chars[start_boundary..end_boundary].iter().collect();
        
        // Clean up the context (remove excessive whitespace, newlines)
        context
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(context_length)
            .collect()
    }

    /// Find word boundary near the given character position
    fn find_char_word_boundary(&self, chars: &[char], position: usize, search_backward: bool) -> usize {
        if position >= chars.len() {
            return chars.len();
        }
        
        if search_backward {
            // Search backward for word boundary
            for i in (0..=position).rev() {
                if i == 0 || chars[i].is_whitespace() || chars[i] == '\n' {
                    return i;
                }
            }
            0
        } else {
            // Search forward for word boundary
            for i in position..chars.len() {
                if chars[i].is_whitespace() || chars[i] == '\n' {
                    return i;
                }
            }
            chars.len()
        }
    }
}

impl Default for LinkExtractor {
    fn default() -> Self {
        Self::new().expect("Failed to create default LinkExtractor")
    }
}

/// Front matter parser using yaml-front-matter library
pub struct FrontMatterParser;

impl FrontMatterParser {
    /// Parse front matter from markdown content using yaml-front-matter library
    /// Returns (metadata, remaining_content)
    pub fn parse(content: &str) -> Result<(ArticleMetadata, String)> {
        // Try to parse with yaml-front-matter
        match YamlFrontMatter::parse(content) {
            Ok(Document { metadata, content: markdown_content }) => {
                // Parse metadata into ArticleMetadata struct
                let metadata: ArticleMetadata = serde_yaml::from_value(metadata)
                    .context("Failed to deserialize front matter metadata")?;
                
                Ok((metadata, markdown_content))
            }
            Err(_) => {
                // No front matter found, return default metadata and full content
                Ok((ArticleMetadata::default(), content.to_string()))
            }
        }
    }

    /// Validate metadata fields
    pub fn validate_metadata(metadata: &ArticleMetadata) -> Result<()> {
        // Validate importance range
        if metadata.importance < 1 || metadata.importance > 5 {
            return Err(anyhow::anyhow!(
                "Importance must be between 1 and 5, got: {}", 
                metadata.importance
            ));
        }

        // Validate title is not empty
        if metadata.title.trim().is_empty() {
            return Err(anyhow::anyhow!("Title cannot be empty"));
        }

        // Validate datetime formats if present
        if let Some(created_at) = &metadata.created_at {
            DateTime::parse_from_rfc3339(created_at)
                .context("Invalid created_at datetime format")?;
        }

        if let Some(updated_at) = &metadata.updated_at {
            DateTime::parse_from_rfc3339(updated_at)
                .context("Invalid updated_at datetime format")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    // Link extraction tests
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
}