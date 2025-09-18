use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use chrono::DateTime;
use regex::Regex;
use yaml_front_matter::{Document, YamlFrontMatter};
use std::collections::{HashMap, HashSet};

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
    pub author_image: Option<String>,
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
            author_image: None,
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

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationErrorType {
    BrokenLink,
    InvalidRelatedArticle,
    MissingMetadata,
    InvalidMetadata,
    CircularReference,
    OrphanedArticle,
}

/// Validation warning types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationWarningType {
    UnusedTag,
    LowImportanceWithManyLinks,
    HighImportanceWithFewLinks,
    MissingBacklinks,
    InconsistentCasing,
}

/// Represents a validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub error_type: ValidationErrorType,
    pub source_article: String,
    pub target_reference: String,
    pub context: Option<String>,
    pub line_number: Option<usize>,
    pub suggestion: Option<String>,
}

/// Represents a validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub warning_type: ValidationWarningType,
    pub source_article: String,
    pub target_reference: Option<String>,
    pub context: Option<String>,
    pub suggestion: Option<String>,
}

/// Summary statistics for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total_articles: usize,
    pub total_links: usize,
    pub broken_links: usize,
    pub invalid_references: usize,
    pub orphaned_articles: usize,
    pub circular_references: usize,
    pub articles_with_errors: usize,
    pub articles_with_warnings: usize,
}

/// Complete validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub validation_date: String,
    pub summary: ValidationSummary,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub article_stats: HashMap<String, ArticleValidationStats>,
}

/// Per-article validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleValidationStats {
    pub outbound_links: usize,
    pub inbound_links: usize,
    pub broken_outbound_links: usize,
    pub invalid_related_articles: usize,
    pub has_errors: bool,
    pub has_warnings: bool,
}

/// Link validation system
pub struct LinkValidator {
    existing_articles: HashSet<String>,
    article_map: HashMap<String, ProcessedArticleRef>,
}

/// Reference to a processed article for validation
#[derive(Debug, Clone)]
pub struct ProcessedArticleRef {
    pub slug: String,
    pub title: String,
    pub metadata: ArticleMetadata,
    pub outbound_links: Vec<ExtractedLink>,
    pub file_path: String,
}

impl LinkValidator {
    /// Create a new link validator with article data
    pub fn new(articles: &[ProcessedArticleRef]) -> Self {
        let existing_articles: HashSet<String> = articles
            .iter()
            .map(|a| a.slug.clone())
            .collect();
        
        let article_map: HashMap<String, ProcessedArticleRef> = articles
            .iter()
            .map(|a| (a.slug.clone(), a.clone()))
            .collect();
        
        Self {
            existing_articles,
            article_map,
        }
    }

    /// Validate all articles and generate a comprehensive report
    pub fn validate_all(&self) -> Result<ValidationReport> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut article_stats = HashMap::new();
        
        // Validate each article
        for article in self.article_map.values() {
            let (article_errors, article_warnings, stats) = self.validate_article(article)?;
            
            errors.extend(article_errors);
            warnings.extend(article_warnings);
            article_stats.insert(article.slug.clone(), stats);
        }
        
        // Generate summary statistics
        let summary = self.generate_summary(&errors, &warnings, &article_stats);
        
        Ok(ValidationReport {
            validation_date: chrono::Utc::now().to_rfc3339(),
            summary,
            errors,
            warnings,
            article_stats,
        })
    }

    /// Validate a single article
    fn validate_article(&self, article: &ProcessedArticleRef) -> Result<(Vec<ValidationError>, Vec<ValidationWarning>, ArticleValidationStats)> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate outbound links
        let mut broken_outbound_links = 0;
        for link in &article.outbound_links {
            if !self.existing_articles.contains(&link.target_slug) {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::BrokenLink,
                    source_article: article.slug.clone(),
                    target_reference: link.target_slug.clone(),
                    context: Some(link.context.clone()),
                    line_number: None, // Could be calculated from position if needed
                    suggestion: self.suggest_similar_article(&link.target_slug),
                });
                broken_outbound_links += 1;
            }
        }
        
        // Validate related_articles in metadata
        let mut invalid_related_articles = 0;
        for related_slug in &article.metadata.related_articles {
            if !self.existing_articles.contains(related_slug) {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::InvalidRelatedArticle,
                    source_article: article.slug.clone(),
                    target_reference: related_slug.clone(),
                    context: Some("front matter related_articles".to_string()),
                    line_number: None,
                    suggestion: self.suggest_similar_article(related_slug),
                });
                invalid_related_articles += 1;
            }
        }
        
        // Check for circular references in related_articles
        for related_slug in &article.metadata.related_articles {
            if let Some(related_article) = self.article_map.get(related_slug) {
                if related_article.metadata.related_articles.contains(&article.slug) {
                    // This is actually a good thing (bidirectional relationship)
                    // But we could warn if there are too many circular references
                    continue;
                }
            }
        }
        
        // Calculate inbound links for this article
        let inbound_links = self.count_inbound_links(&article.slug);
        
        // Generate warnings based on article characteristics
        self.generate_article_warnings(article, inbound_links, &mut warnings);
        
        let stats = ArticleValidationStats {
            outbound_links: article.outbound_links.len(),
            inbound_links,
            broken_outbound_links,
            invalid_related_articles,
            has_errors: !errors.is_empty(),
            has_warnings: !warnings.is_empty(),
        };
        
        Ok((errors, warnings, stats))
    }

    /// Count inbound links to a specific article
    fn count_inbound_links(&self, target_slug: &str) -> usize {
        self.article_map
            .values()
            .map(|article| {
                article.outbound_links
                    .iter()
                    .filter(|link| link.target_slug == target_slug)
                    .count()
            })
            .sum()
    }

    /// Generate warnings for an article based on its characteristics
    fn generate_article_warnings(&self, article: &ProcessedArticleRef, inbound_links: usize, warnings: &mut Vec<ValidationWarning>) {
        // Warning: High importance but few inbound links
        if article.metadata.importance >= 4 && inbound_links < 2 {
            warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::HighImportanceWithFewLinks,
                source_article: article.slug.clone(),
                target_reference: None,
                context: Some(format!("Importance: {}, Inbound links: {}", article.metadata.importance, inbound_links)),
                suggestion: Some("Consider adding more links to this important article or reducing its importance level".to_string()),
            });
        }
        
        // Warning: Low importance but many inbound links
        if article.metadata.importance <= 2 && inbound_links >= 5 {
            warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::LowImportanceWithManyLinks,
                source_article: article.slug.clone(),
                target_reference: None,
                context: Some(format!("Importance: {}, Inbound links: {}", article.metadata.importance, inbound_links)),
                suggestion: Some("Consider increasing the importance level of this well-connected article".to_string()),
            });
        }
        
        // Warning: Article has no inbound or outbound links (orphaned)
        if article.outbound_links.is_empty() && inbound_links == 0 {
            warnings.push(ValidationWarning {
                warning_type: ValidationWarningType::MissingBacklinks,
                source_article: article.slug.clone(),
                target_reference: None,
                context: Some("No inbound or outbound links".to_string()),
                suggestion: Some("Consider adding links to/from this article to integrate it better".to_string()),
            });
        }
    }

    /// Suggest similar articles for broken links using simple string similarity
    fn suggest_similar_article(&self, broken_slug: &str) -> Option<String> {
        let mut best_match = None;
        let mut best_score = 0.0;
        
        for existing_slug in &self.existing_articles {
            let score = self.calculate_similarity(broken_slug, existing_slug);
            if score > best_score && score > 0.5 { // Threshold for suggestions
                best_score = score;
                best_match = Some(existing_slug.clone());
            }
        }
        
        best_match.map(|slug| format!("Did you mean '{}'?", slug))
    }

    /// Calculate simple string similarity (Jaccard similarity on character bigrams)
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        let bigrams1: HashSet<String> = s1.chars()
            .collect::<Vec<_>>()
            .windows(2)
            .map(|w| format!("{}{}", w[0], w[1]))
            .collect();
        
        let bigrams2: HashSet<String> = s2.chars()
            .collect::<Vec<_>>()
            .windows(2)
            .map(|w| format!("{}{}", w[0], w[1]))
            .collect();
        
        if bigrams1.is_empty() && bigrams2.is_empty() {
            return 1.0;
        }
        
        let intersection = bigrams1.intersection(&bigrams2).count();
        let union = bigrams1.union(&bigrams2).count();
        
        intersection as f64 / union as f64
    }

    /// Generate summary statistics
    fn generate_summary(&self, errors: &[ValidationError], warnings: &[ValidationWarning], article_stats: &HashMap<String, ArticleValidationStats>) -> ValidationSummary {
        let total_articles = self.article_map.len();
        let total_links: usize = article_stats.values().map(|s| s.outbound_links).sum();
        
        let broken_links = errors.iter()
            .filter(|e| matches!(e.error_type, ValidationErrorType::BrokenLink))
            .count();
        
        let invalid_references = errors.iter()
            .filter(|e| matches!(e.error_type, ValidationErrorType::InvalidRelatedArticle))
            .count();
        
        let orphaned_articles = warnings.iter()
            .filter(|w| matches!(w.warning_type, ValidationWarningType::MissingBacklinks))
            .count();
        
        let articles_with_errors = article_stats.values()
            .filter(|s| s.has_errors)
            .count();
        
        let articles_with_warnings = article_stats.values()
            .filter(|s| s.has_warnings)
            .count();
        
        ValidationSummary {
            total_articles,
            total_links,
            broken_links,
            invalid_references,
            orphaned_articles,
            circular_references: 0, // TODO: Implement circular reference detection
            articles_with_errors,
            articles_with_warnings,
        }
    }
}

/// Report formatter for validation results
pub struct ValidationReportFormatter;

impl ValidationReportFormatter {
    /// Format validation report as JSON
    pub fn format_json(report: &ValidationReport) -> Result<String> {
        serde_json::to_string_pretty(report)
            .context("Failed to serialize validation report to JSON")
    }

    /// Format validation report for console output
    pub fn format_console(report: &ValidationReport) -> String {
        let mut output = String::new();
        
        // Header
        output.push_str("🔍 Link Validation Report\n");
        output.push_str(&format!("📅 Generated: {}\n\n", report.validation_date));
        
        // Summary
        output.push_str("📊 Summary:\n");
        output.push_str(&format!("   📚 Total articles: {}\n", report.summary.total_articles));
        output.push_str(&format!("   🔗 Total links: {}\n", report.summary.total_links));
        
        if report.summary.broken_links > 0 {
            output.push_str(&format!("   ❌ Broken links: {}\n", report.summary.broken_links));
        } else {
            output.push_str("   ✅ All links valid\n");
        }
        
        if report.summary.invalid_references > 0 {
            output.push_str(&format!("   ⚠️  Invalid references: {}\n", report.summary.invalid_references));
        }
        
        if report.summary.orphaned_articles > 0 {
            output.push_str(&format!("   🏝️  Orphaned articles: {}\n", report.summary.orphaned_articles));
        }
        
        output.push_str(&format!("   📄 Articles with errors: {}\n", report.summary.articles_with_errors));
        output.push_str(&format!("   ⚠️  Articles with warnings: {}\n", report.summary.articles_with_warnings));
        
        // Errors section
        if !report.errors.is_empty() {
            output.push_str("\n❌ Errors:\n");
            for (i, error) in report.errors.iter().enumerate() {
                output.push_str(&format!("{}. ", i + 1));
                output.push_str(&Self::format_error(error));
                output.push('\n');
            }
        }
        
        // Warnings section
        if !report.warnings.is_empty() {
            output.push_str("\n⚠️  Warnings:\n");
            for (i, warning) in report.warnings.iter().enumerate() {
                output.push_str(&format!("{}. ", i + 1));
                output.push_str(&Self::format_warning(warning));
                output.push('\n');
            }
        }
        
        // Article statistics (top problematic articles)
        if !report.article_stats.is_empty() {
            output.push_str("\n📈 Article Statistics:\n");
            
            // Find articles with most issues
            let mut articles_with_issues: Vec<_> = report.article_stats
                .iter()
                .filter(|(_, stats)| stats.has_errors || stats.has_warnings)
                .collect();
            
            articles_with_issues.sort_by(|a, b| {
                let a_issues = a.1.broken_outbound_links + a.1.invalid_related_articles;
                let b_issues = b.1.broken_outbound_links + b.1.invalid_related_articles;
                b_issues.cmp(&a_issues)
            });
            
            for (slug, stats) in articles_with_issues.iter().take(10) {
                output.push_str(&format!("   📄 {}: ", slug));
                
                let mut issue_parts = Vec::new();
                if stats.broken_outbound_links > 0 {
                    issue_parts.push(format!("{} broken links", stats.broken_outbound_links));
                }
                if stats.invalid_related_articles > 0 {
                    issue_parts.push(format!("{} invalid references", stats.invalid_related_articles));
                }
                
                if !issue_parts.is_empty() {
                    output.push_str(&issue_parts.join(", "));
                } else if stats.has_warnings {
                    output.push_str("warnings only");
                }
                
                output.push_str(&format!(" ({} out, {} in)", stats.outbound_links, stats.inbound_links));
                output.push('\n');
            }
        }
        
        // Footer with recommendations
        if report.summary.broken_links > 0 || report.summary.invalid_references > 0 {
            output.push_str("\n💡 Recommendations:\n");
            output.push_str("   • Fix broken links by updating article references\n");
            output.push_str("   • Remove invalid entries from related_articles in front matter\n");
            output.push_str("   • Consider creating missing articles if they are frequently referenced\n");
        }
        
        if report.summary.orphaned_articles > 0 {
            output.push_str("   • Add links to/from orphaned articles to improve connectivity\n");
        }
        
        output
    }

    /// Format a single validation error
    pub fn format_error(error: &ValidationError) -> String {
        let error_type_str = match error.error_type {
            ValidationErrorType::BrokenLink => "🔗 Broken Link",
            ValidationErrorType::InvalidRelatedArticle => "📋 Invalid Related Article",
            ValidationErrorType::MissingMetadata => "📝 Missing Metadata",
            ValidationErrorType::InvalidMetadata => "❌ Invalid Metadata",
            ValidationErrorType::CircularReference => "🔄 Circular Reference",
            ValidationErrorType::OrphanedArticle => "🏝️  Orphaned Article",
        };
        
        let mut formatted = format!("{}: {} → {}", 
            error_type_str, 
            error.source_article, 
            error.target_reference
        );
        
        if let Some(context) = &error.context {
            formatted.push_str(&format!(" ({})", context));
        }
        
        if let Some(suggestion) = &error.suggestion {
            formatted.push_str(&format!(" | 💡 {}", suggestion));
        }
        
        formatted
    }

    /// Format a single validation warning
    pub fn format_warning(warning: &ValidationWarning) -> String {
        let warning_type_str = match warning.warning_type {
            ValidationWarningType::UnusedTag => "🏷️  Unused Tag",
            ValidationWarningType::LowImportanceWithManyLinks => "📈 Low Importance, Many Links",
            ValidationWarningType::HighImportanceWithFewLinks => "📉 High Importance, Few Links",
            ValidationWarningType::MissingBacklinks => "🔗 Missing Backlinks",
            ValidationWarningType::InconsistentCasing => "🔤 Inconsistent Casing",
        };
        
        let mut formatted = format!("{}: {}", warning_type_str, warning.source_article);
        
        if let Some(target) = &warning.target_reference {
            formatted.push_str(&format!(" → {}", target));
        }
        
        if let Some(context) = &warning.context {
            formatted.push_str(&format!(" ({})", context));
        }
        
        if let Some(suggestion) = &warning.suggestion {
            formatted.push_str(&format!(" | 💡 {}", suggestion));
        }
        
        formatted
    }

    /// Generate a compact summary for CI/CD environments
    pub fn format_ci_summary(report: &ValidationReport) -> String {
        let mut output = String::new();
        
        if report.summary.broken_links == 0 && report.summary.invalid_references == 0 {
            output.push_str("✅ All links valid");
        } else {
            output.push_str("❌ Validation failed:");
            if report.summary.broken_links > 0 {
                output.push_str(&format!(" {} broken links", report.summary.broken_links));
            }
            if report.summary.invalid_references > 0 {
                output.push_str(&format!(" {} invalid references", report.summary.invalid_references));
            }
        }
        
        if report.summary.articles_with_warnings > 0 {
            output.push_str(&format!(" ({} warnings)", report.summary.articles_with_warnings));
        }
        
        output
    }

    /// Write validation report to files
    pub fn write_report_files(report: &ValidationReport, output_dir: &std::path::Path) -> Result<()> {
        // Ensure output directory exists
        std::fs::create_dir_all(output_dir)
            .context("Failed to create output directory")?;
        
        // Write JSON report
        let json_path = output_dir.join("validation-report.json");
        let json_content = Self::format_json(report)?;
        std::fs::write(&json_path, json_content)
            .with_context(|| format!("Failed to write JSON report to {:?}", json_path))?;
        
        // Write console report
        let console_path = output_dir.join("validation-report.txt");
        let console_content = Self::format_console(report);
        std::fs::write(&console_path, console_content)
            .with_context(|| format!("Failed to write console report to {:?}", console_path))?;
        
        // Write CI summary
        let ci_path = output_dir.join("validation-summary.txt");
        let ci_content = Self::format_ci_summary(report);
        std::fs::write(&ci_path, ci_content)
            .with_context(|| format!("Failed to write CI summary to {:?}", ci_path))?;
        
        Ok(())
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