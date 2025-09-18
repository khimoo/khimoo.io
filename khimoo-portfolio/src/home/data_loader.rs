use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use yew::prelude::*;

// Data structures matching the generated JSON format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessedArticle {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub metadata: ProcessedMetadata,
    pub file_path: String,
    pub outbound_links: Vec<ProcessedLink>,
    pub inbound_count: usize,
    pub processed_at: String,
}

// Lightweight article data for list display (without full content)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LightweightArticle {
    pub slug: String,
    pub title: String,
    pub summary: Option<String>, // First paragraph or excerpt
    pub metadata: ProcessedMetadata,
    pub file_path: String,
    pub outbound_links: Vec<ProcessedLink>,
    pub inbound_count: usize,
    pub processed_at: String,
}

impl From<ProcessedArticle> for LightweightArticle {
    fn from(article: ProcessedArticle) -> Self {
        // Extract summary from content (first paragraph or first 200 characters)
        let summary = extract_summary(&article.content);
        
        Self {
            slug: article.slug,
            title: article.title,
            summary: Some(summary),
            metadata: article.metadata,
            file_path: article.file_path,
            outbound_links: article.outbound_links,
            inbound_count: article.inbound_count,
            processed_at: article.processed_at,
        }
    }
}

// Extract summary from article content
fn extract_summary(content: &str) -> String {
    // Remove markdown headers and get first paragraph
    let lines: Vec<&str> = content.lines().collect();
    let mut summary_lines = Vec::new();
    let mut found_content = false;
    
    for line in lines {
        let trimmed = line.trim();
        
        // Skip empty lines and headers at the beginning
        if trimmed.is_empty() || trimmed.starts_with('#') {
            if found_content {
                break; // Stop at first empty line or header after content
            }
            continue;
        }
        
        found_content = true;
        summary_lines.push(trimmed);
        
        // Stop after first paragraph or when we have enough content
        if summary_lines.join(" ").len() > 200 {
            break;
        }
    }
    
    let summary = summary_lines.join(" ");
    if summary.len() > 200 {
        format!("{}...", &summary[..197])
    } else {
        summary
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessedMetadata {
    pub title: String,
    pub home_display: bool,
    pub category: Option<String>,
    pub importance: Option<u8>,
    pub related_articles: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub author_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessedLink {
    pub target_slug: String,
    pub link_type: LinkType,
    pub context: String,
    pub position: usize,
    pub original_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LinkType {
    WikiLink,
    MarkdownLink,
    TagReference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArticlesData {
    pub articles: Vec<ProcessedArticle>,
    pub generated_at: String,
    pub total_count: usize,
    pub home_articles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkGraphData {
    pub graph: HashMap<String, GraphNode>,
    pub generated_at: String,
    pub total_connections: usize,
    pub bidirectional_pairs: Option<usize>,
    pub direct_links: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphNode {
    pub connections: Vec<GraphConnection>,
    pub inbound_count: usize,
    pub outbound_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphConnection {
    pub target: String,
    pub connection_type: ConnectionType,
    pub bidirectional: bool,
    pub link_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType {
    DirectLink,
    Bidirectional,
}

// Error types for data loading
#[derive(Debug, Clone, PartialEq)]
pub enum DataLoadError {
    NetworkError(String),
    ParseError(String),
    NotFound(String),
}

impl std::fmt::Display for DataLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataLoadError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            DataLoadError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            DataLoadError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for DataLoadError {}

// DataLoader structure for loading static JSON files
#[derive(Debug, Clone)]
pub struct DataLoader {
    base_url: String,
}

impl DataLoader {
    pub fn new() -> Self {
        // Automatically detect the correct base URL based on the current location
        let base_url = Self::detect_base_url();
        Self { base_url }
    }
    
    // Detect the correct base URL based on the current window location
    fn detect_base_url() -> String {
        if let Some(window) = web_sys::window() {
            if let Some(location) = window.location().pathname().ok() {
                web_sys::console::log_1(&format!("DataLoader: Current pathname: {}", location).into());
                
                // If we're in a subdirectory (like /khimoo.io/), use that as the base
                if location.starts_with("/khimoo.io/") || location.contains("/khimoo.io") {
                    web_sys::console::log_1(&"DataLoader: Detected GitHub Pages subdirectory, using /khimoo.io/data".into());
                    return "/khimoo.io/data".to_string();
                }
            }
            
            // Also check the hostname for additional context
            if let Some(hostname) = window.location().hostname().ok() {
                web_sys::console::log_1(&format!("DataLoader: Current hostname: {}", hostname).into());
                if hostname.contains("github.io") {
                    web_sys::console::log_1(&"DataLoader: Detected GitHub Pages, using /khimoo.io/data".into());
                    return "/khimoo.io/data".to_string();
                }
            }
        }
        
        web_sys::console::log_1(&"DataLoader: Using default base URL: /data".into());
        // Default fallback
        "/data".to_string()
    }

    pub fn with_base_url(base_url: String) -> Self {
        Self { base_url }
    }

    // Load articles data with error handling and fallback
    pub async fn load_articles(&self) -> Result<ArticlesData, DataLoadError> {
        let url = format!("{}/articles.json", self.base_url);
        
        // Debug logging
        web_sys::console::log_1(&format!("DataLoader: Attempting to load articles from: {}", url).into());
        web_sys::console::log_1(&format!("DataLoader: Base URL detected as: {}", self.base_url).into());
        
        match self.fetch_json::<ArticlesData>(&url).await {
            Ok(data) => {
                web_sys::console::log_1(&format!("DataLoader: Successfully loaded {} articles", data.articles.len()).into());
                Ok(data)
            },
            Err(e) => {
                web_sys::console::warn_1(&format!("Failed to load articles data: {}", e).into());
                // Fallback to empty data structure
                Ok(ArticlesData {
                    articles: Vec::new(),
                    generated_at: "1970-01-01T00:00:00Z".to_string(),
                    total_count: 0,
                    home_articles: Vec::new(),
                })
            }
        }
    }

    // Load link graph data with error handling and fallback
    pub async fn load_link_graph(&self) -> Result<LinkGraphData, DataLoadError> {
        let url = format!("{}/link-graph.json", self.base_url);
        
        match self.fetch_json::<LinkGraphData>(&url).await {
            Ok(data) => Ok(data),
            Err(e) => {
                web_sys::console::warn_1(&format!("Failed to load link graph data: {}", e).into());
                // Fallback to empty graph structure
                Ok(LinkGraphData {
                    graph: HashMap::new(),
                    generated_at: "1970-01-01T00:00:00Z".to_string(),
                    total_connections: 0,
                    bidirectional_pairs: Some(0),
                    direct_links: Some(0),
                })
            }
        }
    }

    // Load both articles and link graph data concurrently
    pub async fn load_all_data(&self) -> (Result<ArticlesData, DataLoadError>, Result<LinkGraphData, DataLoadError>) {
        let articles_future = self.load_articles();
        let link_graph_future = self.load_link_graph();
        
        // Use futures to load both concurrently
        let (articles_result, link_graph_result) = futures::join!(articles_future, link_graph_future);
        
        (articles_result, link_graph_result)
    }

    // Load lightweight articles data (without full content)
    pub async fn load_lightweight_articles(&self) -> Result<Vec<LightweightArticle>, DataLoadError> {
        let articles_data = self.load_articles().await?;
        let lightweight_articles = articles_data.articles
            .into_iter()
            .map(LightweightArticle::from)
            .collect();
        Ok(lightweight_articles)
    }

    // Load full article content by slug
    pub async fn load_article_content(&self, slug: &str) -> Result<String, DataLoadError> {
        // For now, we load from the full articles data
        // In a more optimized implementation, we could have separate content files
        let articles_data = self.load_articles().await?;
        
        articles_data.articles
            .into_iter()
            .find(|article| article.slug == slug)
            .map(|article| article.content)
            .ok_or_else(|| DataLoadError::NotFound(format!("Article not found: {}", slug)))
    }

    // Load article by slug (full data)
    pub async fn load_article_by_slug(&self, slug: &str) -> Result<ProcessedArticle, DataLoadError> {
        let articles_data = self.load_articles().await?;
        
        articles_data.articles
            .into_iter()
            .find(|article| article.slug == slug)
            .ok_or_else(|| DataLoadError::NotFound(format!("Article not found: {}", slug)))
    }

    // Generic JSON fetching method
    async fn fetch_json<T>(&self, url: &str) -> Result<T, DataLoadError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(url, &opts)
            .map_err(|e| DataLoadError::NetworkError(format!("Failed to create request: {:?}", e)))?;

        let window = web_sys::window()
            .ok_or_else(|| DataLoadError::NetworkError("No window object".to_string()))?;

        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| DataLoadError::NetworkError(format!("Fetch failed: {:?}", e)))?;

        let resp: Response = resp_value
            .dyn_into()
            .map_err(|e| DataLoadError::NetworkError(format!("Invalid response: {:?}", e)))?;

        if !resp.ok() {
            return Err(DataLoadError::NotFound(format!(
                "HTTP {}: {}",
                resp.status(),
                resp.status_text()
            )));
        }

        let json = JsFuture::from(resp.json().map_err(|e| {
            DataLoadError::ParseError(format!("Failed to get JSON: {:?}", e))
        })?)
        .await
        .map_err(|e| DataLoadError::ParseError(format!("Failed to parse JSON: {:?}", e)))?;

        let data: T = serde_wasm_bindgen::from_value(json)
            .map_err(|e| DataLoadError::ParseError(format!("Failed to deserialize: {:?}", e)))?;

        Ok(data)
    }
}

impl Default for DataLoader {
    fn default() -> Self {
        Self::new()
    }
}

// Hook for using DataLoader in Yew components
#[hook]
pub fn use_data_loader() -> UseStateHandle<Option<DataLoader>> {
    use_state(|| Some(DataLoader::new()))
}

// Hook for loading articles data
#[hook]
pub fn use_articles_data() -> (UseStateHandle<Option<ArticlesData>>, UseStateHandle<bool>, UseStateHandle<Option<DataLoadError>>) {
    let data = use_state(|| None);
    let loading = use_state(|| true);
    let error = use_state(|| None);

    {
        let data = data.clone();
        let loading = loading.clone();
        let error = error.clone();
        
        use_effect_with((), move |_| {
            let data = data.clone();
            let loading = loading.clone();
            let error = error.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                let loader = DataLoader::new();
                match loader.load_articles().await {
                    Ok(articles_data) => {
                        data.set(Some(articles_data));
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
                loading.set(false);
            });
            
            || {}
        });
    }

    (data, loading, error)
}

// Hook for loading link graph data
#[hook]
pub fn use_link_graph_data() -> (UseStateHandle<Option<LinkGraphData>>, UseStateHandle<bool>, UseStateHandle<Option<DataLoadError>>) {
    let data = use_state(|| None);
    let loading = use_state(|| true);
    let error = use_state(|| None);

    {
        let data = data.clone();
        let loading = loading.clone();
        let error = error.clone();
        
        use_effect_with((), move |_| {
            let data = data.clone();
            let loading = loading.clone();
            let error = error.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                let loader = DataLoader::new();
                match loader.load_link_graph().await {
                    Ok(link_graph_data) => {
                        data.set(Some(link_graph_data));
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
                loading.set(false);
            });
            
            || {}
        });
    }

    (data, loading, error)
}

// Hook for loading lightweight articles (for list display)
#[hook]
pub fn use_lightweight_articles() -> (UseStateHandle<Option<Vec<LightweightArticle>>>, UseStateHandle<bool>, UseStateHandle<Option<DataLoadError>>) {
    let data = use_state(|| None);
    let loading = use_state(|| true);
    let error = use_state(|| None);

    {
        let data = data.clone();
        let loading = loading.clone();
        let error = error.clone();
        
        use_effect_with((), move |_| {
            let data = data.clone();
            let loading = loading.clone();
            let error = error.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                let loader = DataLoader::new();
                match loader.load_lightweight_articles().await {
                    Ok(lightweight_articles) => {
                        data.set(Some(lightweight_articles));
                        error.set(None);
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
                loading.set(false);
            });
            
            || {}
        });
    }

    (data, loading, error)
}

// Hook for loading a specific article by slug (with caching)
#[hook]
pub fn use_article_content(slug: Option<String>) -> (UseStateHandle<Option<ProcessedArticle>>, UseStateHandle<bool>, UseStateHandle<Option<DataLoadError>>) {
    let data = use_state(|| None);
    let loading = use_state(|| false);
    let error = use_state(|| None);

    {
        let data = data.clone();
        let loading = loading.clone();
        let error = error.clone();
        
        use_effect_with(slug.clone(), move |slug| {
            if let Some(slug) = slug {
                let data = data.clone();
                let loading = loading.clone();
                let error = error.clone();
                let slug = slug.clone();
                
                loading.set(true);
                error.set(None);
                
                wasm_bindgen_futures::spawn_local(async move {
                    let loader = DataLoader::new();
                    match loader.load_article_by_slug(&slug).await {
                        Ok(article) => {
                            data.set(Some(article));
                            error.set(None);
                        }
                        Err(e) => {
                            error.set(Some(e));
                        }
                    }
                    loading.set(false);
                });
            } else {
                data.set(None);
                loading.set(false);
                error.set(None);
            }
            
            || {}
        });
    }

    (data, loading, error)
}