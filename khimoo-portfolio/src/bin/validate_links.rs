use khimoo_portfolio::article_processing::{
    FrontMatterParser, LinkExtractor, LinkValidator, ProcessedArticleRef,
    ValidationReportFormatter, ValidationReport
};
use anyhow::{Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "validate-links")]
#[command(about = "Validate links and references in articles")]
struct Args {
    /// Articles directory path
    #[arg(short, long, default_value = "articles")]
    articles_dir: PathBuf,
    
    /// Output directory for validation reports
    #[arg(short, long, default_value = "data")]
    output_dir: PathBuf,
    
    /// Output format: console, json, ci, or all
    #[arg(short, long, default_value = "console")]
    format: String,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Exit with error code if validation fails
    #[arg(long)]
    fail_on_error: bool,
    
    /// Only show errors, suppress warnings
    #[arg(long)]
    errors_only: bool,
    
    /// Write reports to files even in console mode
    #[arg(long)]
    write_files: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    if args.verbose {
        println!("🔍 Validating links in {:?}", args.articles_dir);
        println!("📁 Output directory: {:?}", args.output_dir);
        println!("📋 Format: {}", args.format);
    }
    
    let validator = LinkValidationTool::new(
        args.articles_dir,
        args.output_dir,
        args.verbose,
        args.errors_only,
    );
    
    let report = validator.validate_all_articles()
        .context("Failed to validate articles")?;
    
    // Output the report in the requested format
    validator.output_report(&report, &args.format, args.write_files)
        .context("Failed to output validation report")?;
    
    // Exit with error code if validation failed and fail_on_error is set
    if args.fail_on_error && (report.summary.broken_links > 0 || report.summary.invalid_references > 0) {
        std::process::exit(1);
    }
    
    Ok(())
}

pub struct LinkValidationTool {
    articles_dir: PathBuf,
    output_dir: PathBuf,
    verbose: bool,
    errors_only: bool,
    link_extractor: LinkExtractor,
}

impl LinkValidationTool {
    pub fn new(articles_dir: PathBuf, output_dir: PathBuf, verbose: bool, errors_only: bool) -> Self {
        Self {
            articles_dir,
            output_dir,
            verbose,
            errors_only,
            link_extractor: LinkExtractor::new().expect("Failed to create LinkExtractor"),
        }
    }

    pub fn validate_all_articles(&self) -> Result<ValidationReport> {
        if self.verbose {
            println!("📚 Loading articles from {:?}", self.articles_dir);
        }
        
        // Load all articles
        let articles = self.load_articles()
            .context("Failed to load articles")?;
        
        if self.verbose {
            println!("✅ Loaded {} articles", articles.len());
            println!("🔍 Running validation...");
        }
        
        // Create validator and run validation
        let validator = LinkValidator::new(&articles);
        let report = validator.validate_all()
            .context("Failed to run validation")?;
        
        if self.verbose {
            println!("✅ Validation complete");
            self.print_validation_summary(&report);
        }
        
        Ok(report)
    }

    fn load_articles(&self) -> Result<Vec<ProcessedArticleRef>> {
        let mut articles = Vec::new();
        
        for entry in WalkDir::new(&self.articles_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        {
            match self.load_article_file(entry.path()) {
                Ok(article) => {
                    if self.verbose {
                        println!("   📄 Loaded: {} - '{}'", 
                            entry.path().display(), 
                            article.title
                        );
                    }
                    articles.push(article);
                }
                Err(e) => {
                    eprintln!("❌ Error loading {}: {}", entry.path().display(), e);
                    return Err(e);
                }
            }
        }
        
        Ok(articles)
    }

    fn load_article_file(&self, file_path: &Path) -> Result<ProcessedArticleRef> {
        // Read file content
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {:?}", file_path))?;

        // Parse front matter
        let (metadata, markdown_content) = FrontMatterParser::parse(&content)
            .with_context(|| format!("Failed to parse front matter in: {:?}", file_path))?;

        // Validate metadata
        FrontMatterParser::validate_metadata(&metadata)
            .with_context(|| format!("Invalid metadata in: {:?}", file_path))?;

        // Extract links from content
        let outbound_links = self.link_extractor.extract_links(&markdown_content);

        // Generate slug from file path
        let slug = self.generate_slug(file_path);

        Ok(ProcessedArticleRef {
            slug,
            title: metadata.title.clone(),
            metadata,
            outbound_links,
            file_path: file_path.to_string_lossy().to_string(),
        })
    }

    fn generate_slug(&self, file_path: &Path) -> String {
        file_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string()
            .to_lowercase()
            .replace(' ', "-")
    }

    fn print_validation_summary(&self, report: &ValidationReport) {
        println!("📊 Validation Summary:");
        println!("   📚 Articles: {}", report.summary.total_articles);
        println!("   🔗 Links: {}", report.summary.total_links);
        
        if report.summary.broken_links > 0 {
            println!("   ❌ Broken links: {}", report.summary.broken_links);
        }
        
        if report.summary.invalid_references > 0 {
            println!("   ⚠️  Invalid references: {}", report.summary.invalid_references);
        }
        
        if report.summary.orphaned_articles > 0 {
            println!("   🏝️  Orphaned articles: {}", report.summary.orphaned_articles);
        }
        
        println!("   📄 Articles with errors: {}", report.summary.articles_with_errors);
        
        if !self.errors_only {
            println!("   ⚠️  Articles with warnings: {}", report.summary.articles_with_warnings);
        }
    }

    pub fn output_report(&self, report: &ValidationReport, format: &str, write_files: bool) -> Result<()> {
        match format.to_lowercase().as_str() {
            "console" => {
                let console_output = if self.errors_only {
                    self.format_console_errors_only(report)
                } else {
                    ValidationReportFormatter::format_console(report)
                };
                println!("{}", console_output);
                
                if write_files {
                    self.write_report_files(report)?;
                }
            }
            "json" => {
                let json_output = ValidationReportFormatter::format_json(report)?;
                println!("{}", json_output);
            }
            "ci" => {
                let ci_output = ValidationReportFormatter::format_ci_summary(report);
                println!("{}", ci_output);
            }
            "all" => {
                // Write all formats to files
                self.write_report_files(report)?;
                
                // Also output console format
                let console_output = if self.errors_only {
                    self.format_console_errors_only(report)
                } else {
                    ValidationReportFormatter::format_console(report)
                };
                println!("{}", console_output);
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid format: {}. Use console, json, ci, or all", format));
            }
        }
        
        Ok(())
    }

    fn format_console_errors_only(&self, report: &ValidationReport) -> String {
        let mut output = String::new();
        
        // Header
        output.push_str("🔍 Link Validation Report (Errors Only)\n");
        output.push_str(&format!("📅 Generated: {}\n\n", report.validation_date));
        
        // Summary (errors only)
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
        
        output.push_str(&format!("   📄 Articles with errors: {}\n", report.summary.articles_with_errors));
        
        // Errors section only
        if !report.errors.is_empty() {
            output.push_str("\n❌ Errors:\n");
            for (i, error) in report.errors.iter().enumerate() {
                output.push_str(&format!("{}. ", i + 1));
                output.push_str(&ValidationReportFormatter::format_error(error));
                output.push('\n');
            }
        }
        
        // Recommendations for errors only
        if report.summary.broken_links > 0 || report.summary.invalid_references > 0 {
            output.push_str("\n💡 Recommendations:\n");
            output.push_str("   • Fix broken links by updating article references\n");
            output.push_str("   • Remove invalid entries from related_articles in front matter\n");
            output.push_str("   • Consider creating missing articles if they are frequently referenced\n");
        }
        
        output
    }

    fn write_report_files(&self, report: &ValidationReport) -> Result<()> {
        // Create output directory
        std::fs::create_dir_all(&self.output_dir)
            .context("Failed to create output directory")?;
        
        ValidationReportFormatter::write_report_files(report, &self.output_dir)
            .context("Failed to write report files")?;
        
        if self.verbose {
            println!("📄 Reports written to {:?}", self.output_dir);
            println!("   • validation-report.json");
            println!("   • validation-report.txt");
            println!("   • validation-summary.txt");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_links_tool_basic() -> Result<()> {
        // Create temporary directory with test articles
        let temp_dir = TempDir::new()?;
        let articles_dir = temp_dir.path().join("articles");
        let output_dir = temp_dir.path().join("data");
        
        fs::create_dir_all(&articles_dir)?;
        
        // Create test articles
        fs::write(
            articles_dir.join("article1.md"),
            r#"---
title: "Article 1"
home_display: true
related_articles: ["article2", "missing"]
---

# Article 1

This links to [[article2]] and [[nonexistent]].
"#,
        )?;
        
        fs::write(
            articles_dir.join("article2.md"),
            r#"---
title: "Article 2"
---

# Article 2

This is a valid article.
"#,
        )?;
        
        // Create validation tool
        let tool = LinkValidationTool::new(articles_dir, output_dir, false, false);
        
        // Run validation
        let report = tool.validate_all_articles()?;
        
        // Check results
        assert_eq!(report.summary.total_articles, 2);
        assert_eq!(report.summary.broken_links, 1); // [[nonexistent]]
        assert_eq!(report.summary.invalid_references, 1); // "missing" in related_articles
        assert_eq!(report.errors.len(), 2);
        
        Ok(())
    }

    #[test]
    fn test_validate_links_tool_no_errors() -> Result<()> {
        // Create temporary directory with valid test articles
        let temp_dir = TempDir::new()?;
        let articles_dir = temp_dir.path().join("articles");
        let output_dir = temp_dir.path().join("data");
        
        fs::create_dir_all(&articles_dir)?;
        
        // Create valid test articles
        fs::write(
            articles_dir.join("article1.md"),
            r#"---
title: "Article 1"
related_articles: ["article2"]
---

# Article 1

This links to [[article2]].
"#,
        )?;
        
        fs::write(
            articles_dir.join("article2.md"),
            r#"---
title: "Article 2"
---

# Article 2

This links back to [Article 1](article1).
"#,
        )?;
        
        // Create validation tool
        let tool = LinkValidationTool::new(articles_dir, output_dir, false, false);
        
        // Run validation
        let report = tool.validate_all_articles()?;
        
        // Check results
        assert_eq!(report.summary.total_articles, 2);
        assert_eq!(report.summary.broken_links, 0);
        assert_eq!(report.summary.invalid_references, 0);
        assert_eq!(report.errors.len(), 0);
        
        Ok(())
    }

    #[test]
    fn test_output_formats() -> Result<()> {
        // Create minimal test setup
        let temp_dir = TempDir::new()?;
        let articles_dir = temp_dir.path().join("articles");
        let output_dir = temp_dir.path().join("data");
        
        fs::create_dir_all(&articles_dir)?;
        fs::write(
            articles_dir.join("test.md"),
            "---\ntitle: Test\n---\n# Test\n",
        )?;
        
        let tool = LinkValidationTool::new(articles_dir, output_dir.clone(), false, false);
        let report = tool.validate_all_articles()?;
        
        // Test JSON format
        tool.output_report(&report, "json", false)?;
        
        // Test CI format
        tool.output_report(&report, "ci", false)?;
        
        // Test console format with file writing
        tool.output_report(&report, "console", true)?;
        
        // Check that files were written
        assert!(output_dir.join("validation-report.json").exists());
        assert!(output_dir.join("validation-report.txt").exists());
        assert!(output_dir.join("validation-summary.txt").exists());
        
        Ok(())
    }
}