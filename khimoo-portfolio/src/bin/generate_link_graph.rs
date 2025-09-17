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
#[command(name = "generate-link-graph")]
#[command(about = "Generate link graph data from processed articles")]
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
    
    /// Only generate link graph (don't process articles)
    #[arg(long)]
    graph_only: bool,
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
pub struct LinkGraphData {
    pub graph: HashMap<String, GraphNode>,
    pub generated_at: String,
    pub total_connections: usize,
    pub bidirectional_pairs: usize,
    pub direct_links: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub connections: Vec<GraphConnection>,
    pub inbound_count: usize,
    pub outbound_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConnection {
    pub target: String,
    pub connection_type: ConnectionType,
    pub bidirectional: bool,
    pub link_count: usize, // Number of actual links (for duplicate detection)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    DirectLink,
    Bidirectional,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    if args.verbose {
        println!("🕸️  Generating link graph from {:?}", args.articles_dir);
        println!("📁 Output directory: {:?}", args.output_dir);
        println!("🔧 Graph only mode: {}", args.graph_only);
    }
    
    let generator = LinkGraphGenerator::new(args.articles_dir, args.output_dir, args.verbose);
    
    if args.graph_only {
        generator.generate_from_existing_data()
    } else {
        generator.generate_from_articles()
    }
}

pub struct LinkGraphGenerator {
    articles_dir: PathBuf,
    output_dir: PathBuf,
    verbose: bool,
    link_extractor: LinkExtractor,
}

impl LinkGraphGenerator {
    pub fn new(articles_dir: PathBuf, output_dir: PathBuf, verbose: bool) -> Self {
        Self {
            articles_dir,
            output_dir,
            verbose,
            link_extractor: LinkExtractor::new().expect("Failed to create LinkExtractor"),
        }
    }

    pub fn generate_from_articles(&self) -> Result<()> {
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
        
        // Build link graph
        let link_graph = self.build_link_graph(&articles_with_counts)
            .context("Failed to build link graph")?;
        
        // Write link graph data
        self.write_link_graph_data(&link_graph)
            .context("Failed to write link graph data")?;
        
        println!("✅ Successfully generated link graph for {} articles", articles_with_counts.len());
        
        // Display link graph summary
        self.display_link_graph_summary(&link_graph);
        
        Ok(())
    }

    pub fn generate_from_existing_data(&self) -> Result<()> {
        // Try to read existing articles.json
        let articles_path = self.output_dir.join("articles.json");
        
        if !articles_path.exists() {
            return Err(anyhow::anyhow!(
                "articles.json not found at {:?}. Run without --graph-only first.", 
                articles_path
            ));
        }
        
        if self.verbose {
            println!("📖 Reading existing articles data from {:?}", articles_path);
        }
        
        let articles_json = std::fs::read_to_string(&articles_path)
            .with_context(|| format!("Failed to read articles data from {:?}", articles_path))?;
        
        let articles_data: serde_json::Value = serde_json::from_str(&articles_json)
            .context("Failed to parse articles JSON")?;
        
        let articles: Vec<ProcessedArticle> = serde_json::from_value(
            articles_data["articles"].clone()
        ).context("Failed to deserialize articles from JSON")?;
        
        if self.verbose {
            println!("📚 Loaded {} articles from existing data", articles.len());
        }
        
        // Build link graph
        let link_graph = self.build_link_graph(&articles)
            .context("Failed to build link graph")?;
        
        // Write link graph data
        self.write_link_graph_data(&link_graph)
            .context("Failed to write link graph data")?;
        
        println!("✅ Successfully generated link graph from existing data");
        
        // Display link graph summary
        self.display_link_graph_summary(&link_graph);
        
        Ok(())
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

    fn build_link_graph(&self, articles: &[ProcessedArticle]) -> Result<LinkGraphData> {
        let mut graph = HashMap::new();
        let article_slugs: std::collections::HashSet<_> = 
            articles.iter().map(|a| &a.slug).collect();
        
        // First pass: create nodes and direct connections
        for article in articles {
            let mut connections = Vec::new();
            let mut connection_counts: HashMap<String, usize> = HashMap::new();
            
            // Count links to each target (for duplicate detection)
            for link in &article.outbound_links {
                if article_slugs.contains(&link.target_slug) {
                    *connection_counts.entry(link.target_slug.clone()).or_insert(0) += 1;
                }
            }
            
            // Create connections with link counts
            for (target_slug, link_count) in connection_counts {
                connections.push(GraphConnection {
                    target: target_slug,
                    connection_type: ConnectionType::DirectLink,
                    bidirectional: false,
                    link_count,
                });
            }
            
            graph.insert(article.slug.clone(), GraphNode {
                connections,
                inbound_count: article.inbound_count,
                outbound_count: article.outbound_links.len(),
            });
        }
        
        // Second pass: detect bidirectional links
        let mut bidirectional_pairs = std::collections::HashSet::new();
        
        for (source_slug, source_node) in &graph {
            for connection in &source_node.connections {
                let target_slug = &connection.target;
                
                // Check if target also links back to source
                if let Some(target_node) = graph.get(target_slug) {
                    let has_backlink = target_node.connections
                        .iter()
                        .any(|c| c.target == *source_slug);
                    
                    if has_backlink {
                        // Create a canonical pair (smaller slug first) to avoid duplicates
                        let pair = if source_slug < target_slug {
                            (source_slug.clone(), target_slug.clone())
                        } else {
                            (target_slug.clone(), source_slug.clone())
                        };
                        bidirectional_pairs.insert(pair);
                    }
                }
            }
        }
        
        // Third pass: update bidirectional connections
        for (slug1, slug2) in &bidirectional_pairs {
            // Update connections in both nodes
            if let Some(node1) = graph.get_mut(slug1) {
                for connection in &mut node1.connections {
                    if connection.target == *slug2 {
                        connection.connection_type = ConnectionType::Bidirectional;
                        connection.bidirectional = true;
                    }
                }
            }
            
            if let Some(node2) = graph.get_mut(slug2) {
                for connection in &mut node2.connections {
                    if connection.target == *slug1 {
                        connection.connection_type = ConnectionType::Bidirectional;
                        connection.bidirectional = true;
                    }
                }
            }
        }
        
        let total_connections = graph.values()
            .map(|node| node.connections.len())
            .sum();
        
        let direct_links = graph.values()
            .flat_map(|node| &node.connections)
            .filter(|conn| !conn.bidirectional)
            .count();
        
        if self.verbose {
            println!("🕸️  Built link graph with {} nodes and {} connections", 
                graph.len(), total_connections);
            println!("   🔗 Bidirectional pairs: {}", bidirectional_pairs.len());
            println!("   ➡️  Direct links: {}", direct_links);
        }
        
        Ok(LinkGraphData {
            graph,
            generated_at: Utc::now().to_rfc3339(),
            total_connections,
            bidirectional_pairs: bidirectional_pairs.len(),
            direct_links,
        })
    }

    fn write_link_graph_data(&self, link_graph: &LinkGraphData) -> Result<()> {
        let output_path = self.output_dir.join("link-graph.json");
        let json = serde_json::to_string_pretty(link_graph)
            .context("Failed to serialize link graph data")?;
        
        std::fs::write(&output_path, json)
            .with_context(|| format!("Failed to write link graph data to {:?}", output_path))?;
        
        if self.verbose {
            println!("🕸️  Written link graph data to: {:?}", output_path);
        }
        
        Ok(())
    }

    fn display_link_graph_summary(&self, link_graph: &LinkGraphData) {
        println!("\n🕸️  Link Graph Summary:");
        println!("   📊 Total nodes: {}", link_graph.graph.len());
        println!("   🔗 Total connections: {}", link_graph.total_connections);
        println!("   ↔️  Bidirectional pairs: {}", link_graph.bidirectional_pairs);
        println!("   ➡️  Direct links: {}", link_graph.direct_links);
        
        // Find most connected nodes
        let mut node_connections: Vec<_> = link_graph.graph
            .iter()
            .map(|(slug, node)| (slug, node.connections.len(), node.inbound_count))
            .collect();
        
        node_connections.sort_by(|a, b| b.1.cmp(&a.1));
        
        if !node_connections.is_empty() {
            println!("   📈 Most connected nodes:");
            for (slug, outbound, inbound) in node_connections.iter().take(5) {
                println!("      {}: {} out, {} in", slug, outbound, inbound);
            }
        }
        
        // Find nodes with duplicate links
        let nodes_with_duplicates: Vec<_> = link_graph.graph
            .iter()
            .filter_map(|(slug, node)| {
                let max_link_count = node.connections
                    .iter()
                    .map(|c| c.link_count)
                    .max()
                    .unwrap_or(0);
                
                if max_link_count > 1 {
                    Some((slug, max_link_count))
                } else {
                    None
                }
            })
            .collect();
        
        if !nodes_with_duplicates.is_empty() {
            println!("   🔄 Nodes with duplicate links:");
            for (slug, max_count) in nodes_with_duplicates {
                println!("      {}: max {} links to same target", slug, max_count);
            }
        }
        
        // Find isolated nodes
        let isolated_nodes: Vec<_> = link_graph.graph
            .iter()
            .filter(|(_, node)| node.connections.is_empty() && node.inbound_count == 0)
            .map(|(slug, _)| slug)
            .collect();
        
        if !isolated_nodes.is_empty() {
            println!("   🏝️  Isolated nodes: {}", isolated_nodes.len());
            for slug in isolated_nodes.iter().take(3) {
                println!("      {}", slug);
            }
            if isolated_nodes.len() > 3 {
                println!("      ... and {} more", isolated_nodes.len() - 3);
            }
        }
    }
}