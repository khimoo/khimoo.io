use khimoo_portfolio::article_processing::{
    FrontMatterParser, ArticleMetadata, LinkExtractor, ExtractedLink
};
use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use chrono::Utc;

#[derive(Parser)]
#[command(name = "process-articles")]
#[command(about = "Process articles and generate static data")]
struct Args {
    /// Articles directory path
    #[arg(short, long, default_value = "articles")]
    articles_dir: PathBuf,
    
    /// Output directory for generated data
    #[arg(short, long, default_value = "data")]
    output_dir: PathBuf,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Enable parallel processing
    #[arg(short, long)]
    parallel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedArticle {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub metadata: ArticleMetadata,
    pub file_path: String,
    pub outbound_links: Vec<ExtractedLink>,
    pub inbound_count: usize,

    pub processed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticlesData {
    pub articles: Vec<ProcessedArticle>,
    pub generated_at: String,
    pub total_count: usize,
    pub home_articles: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    if args.verbose {
        println!("🔄 Processing articles from {:?}", args.articles_dir);
        println!("📁 Output directory: {:?}", args.output_dir);
        println!("⚡ Parallel processing: {}", args.parallel);
    }
    
    let processor = ArticleProcessor::new(args.articles_dir, args.output_dir, args.verbose);
    
    if args.parallel {
        processor.process_all_articles_parallel()
    } else {
        processor.process_all_articles_sequential()
    }
}

pub struct ArticleProcessor {
    articles_dir: PathBuf,
    output_dir: PathBuf,
    verbose: bool,
    link_extractor: LinkExtractor,
}

impl ArticleProcessor {
    pub fn new(articles_dir: PathBuf, output_dir: PathBuf, verbose: bool) -> Self {
        Self {
            articles_dir,
            output_dir,
            verbose,
            link_extractor: LinkExtractor::new().expect("Failed to create LinkExtractor"),
        }
    }

    pub fn process_all_articles_sequential(&self) -> Result<()> {
        // Create output directory
        std::fs::create_dir_all(&self.output_dir)
            .context("Failed to create output directory")?;
        
        // Load and parse all articles
        let articles = self.load_and_parse_articles()
            .context("Failed to load articles")?;
        
        if self.verbose {
            println!("📚 Found {} articles", articles.len());
        }
        
        // Calculate inbound link counts
        let articles_with_counts = self.calculate_inbound_counts(articles)
            .context("Failed to calculate inbound counts")?;
        
        // Write output files
        self.write_articles_data(&articles_with_counts)
            .context("Failed to write articles data")?;
        
        println!("✅ Successfully processed {} articles", articles_with_counts.len());
        
        // Display summary
        self.display_summary(&articles_with_counts);
        
        Ok(())
    }

    pub fn process_all_articles_parallel(&self) -> Result<()> {
        // For now, fall back to sequential processing
        // TODO: Implement parallel processing with rayon
        if self.verbose {
            println!("⚠️  Parallel processing not yet implemented, using sequential");
        }
        self.process_all_articles_sequential()
    }

    fn load_and_parse_articles(&self) -> Result<Vec<ProcessedArticle>> {
        let mut articles = Vec::new();
        
        for entry in WalkDir::new(&self.articles_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        {
            match self.process_article_file(entry.path()) {
                Ok(article) => {
                    if self.verbose {
                        println!("✅ Processed: {} - '{}'", 
                            entry.path().display(), 
                            article.title
                        );
                    }
                    articles.push(article);
                }
                Err(e) => {
                    eprintln!("❌ Error processing {}: {}", entry.path().display(), e);
                    return Err(e);
                }
            }
        }
        
        Ok(articles)
    }

    fn process_article_file(&self, file_path: &Path) -> Result<ProcessedArticle> {
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

        if self.verbose {
            println!("   📝 Title: {}", metadata.title);
            println!("   🆔 Slug: {}", slug);
            println!("   🏠 Home display: {}", metadata.home_display);
            if let Some(category) = &metadata.category {
                println!("   📂 Category: {}", category);
            }
            println!("   ⭐ Importance: {}", metadata.importance);
            if !outbound_links.is_empty() {
                println!("   🔗 Outbound links: {}", outbound_links.len());
                for link in &outbound_links {
                    println!("      → {} ({})", link.target_slug, 
                        match link.link_type {
                            khimoo_portfolio::LinkType::WikiLink => "wiki",
                            khimoo_portfolio::LinkType::MarkdownLink => "markdown",
                        }
                    );
                }
            }
            if !metadata.tags.is_empty() {
                println!("   🏷️  Tags: {:?}", metadata.tags);
            }
        }

        Ok(ProcessedArticle {
            slug,
            title: metadata.title.clone(),
            content: markdown_content,
            metadata,
            file_path: file_path.to_string_lossy().to_string(),
            outbound_links,
            inbound_count: 0, // Will be calculated later
            processed_at: Utc::now().to_rfc3339(),
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

    fn calculate_inbound_counts(&self, mut articles: Vec<ProcessedArticle>) -> Result<Vec<ProcessedArticle>> {
        // Create a map of slug -> index for quick lookup
        let slug_to_index: HashMap<String, usize> = articles
            .iter()
            .enumerate()
            .map(|(i, article)| (article.slug.clone(), i))
            .collect();

        // Count inbound links by collecting them first
        let mut inbound_counts: HashMap<String, usize> = HashMap::new();
        
        for article in &articles {
            for link in &article.outbound_links {
                if slug_to_index.contains_key(&link.target_slug) {
                    *inbound_counts.entry(link.target_slug.clone()).or_insert(0) += 1;
                }
            }
        }

        // Apply the counts
        for article in &mut articles {
            article.inbound_count = inbound_counts.get(&article.slug).copied().unwrap_or(0);
        }

        Ok(articles)
    }

    fn write_articles_data(&self, articles: &[ProcessedArticle]) -> Result<()> {
        let articles_data = ArticlesData {
            articles: articles.to_vec(),
            generated_at: Utc::now().to_rfc3339(),
            total_count: articles.len(),
            home_articles: articles
                .iter()
                .filter(|a| a.metadata.home_display)
                .map(|a| a.slug.clone())
                .collect(),
        };
        
        let output_path = self.output_dir.join("articles.json");
        let json = serde_json::to_string_pretty(&articles_data)
            .context("Failed to serialize articles data")?;
        
        std::fs::write(&output_path, json)
            .with_context(|| format!("Failed to write articles data to {:?}", output_path))?;
        
        if self.verbose {
            println!("📄 Written articles data to: {:?}", output_path);
        }
        
        Ok(())
    }

    fn display_summary(&self, articles: &[ProcessedArticle]) {
        println!("\n📊 Processing Summary:");
        println!("   📚 Total articles: {}", articles.len());
        
        let home_articles = articles.iter().filter(|a| a.metadata.home_display).count();
        println!("   🏠 Home display articles: {}", home_articles);
        
        let total_links: usize = articles.iter().map(|a| a.outbound_links.len()).sum();
        println!("   🔗 Total outbound links: {}", total_links);
        
        let articles_with_inbound: usize = articles.iter().filter(|a| a.inbound_count > 0).count();
        println!("   📥 Articles with inbound links: {}", articles_with_inbound);
        
        // Category breakdown
        let mut categories: HashMap<String, usize> = HashMap::new();
        for article in articles {
            let category = article.metadata.category.as_deref().unwrap_or("uncategorized");
            *categories.entry(category.to_string()).or_insert(0) += 1;
        }
        
        if !categories.is_empty() {
            println!("   📂 Categories:");
            for (category, count) in categories {
                println!("      {}: {}", category, count);
            }
        }
        
        // Tag statistics
        let mut all_tags: HashMap<String, usize> = HashMap::new();
        for article in articles {
            for tag in &article.metadata.tags {
                *all_tags.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        
        if !all_tags.is_empty() {
            println!("   🏷️  Most common tags:");
            let mut tag_counts: Vec<_> = all_tags.into_iter().collect();
            tag_counts.sort_by(|a, b| b.1.cmp(&a.1));
            for (tag, count) in tag_counts.into_iter().take(5) {
                println!("      {}: {}", tag, count);
            }
        }
        
        // Link validation preview
        let existing_slugs: std::collections::HashSet<_> = articles.iter().map(|a| &a.slug).collect();
        let mut broken_links = 0;
        
        for article in articles {
            for link in &article.outbound_links {
                if !existing_slugs.contains(&link.target_slug) {
                    broken_links += 1;
                }
            }
        }
        
        if broken_links > 0 {
            println!("   ⚠️  Broken links detected: {} (run validate-links for details)", broken_links);
        } else {
            println!("   ✅ All links valid");
        }
    }
}