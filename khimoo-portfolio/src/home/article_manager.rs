use super::data_loader::{ArticlesData, LinkGraphData, ProcessedArticle, LightweightArticle, DataLoadError, DataLoader};
use super::types::{NodeId, NodeContent, Position, NodeRegistry, AUTHOR_NODE_ID, ConnectionLineType};
use std::collections::HashMap;
use yew::prelude::*;

// ArticleManager for managing article data and integration with physics system
#[derive(Debug, Clone)]
pub struct ArticleManager {
    articles: HashMap<String, ProcessedArticle>,
    lightweight_articles: HashMap<String, LightweightArticle>, // For memory efficiency
    home_articles: Vec<String>,
    link_graph: HashMap<String, Vec<String>>, // slug -> connected slugs
    slug_to_node_id: HashMap<String, NodeId>,
    node_id_to_slug: HashMap<NodeId, String>,
    next_node_id: u32,
    content_cache: HashMap<String, String>, // slug -> full content cache
}

impl ArticleManager {
    pub fn new() -> Self {
        Self {
            articles: HashMap::new(),
            lightweight_articles: HashMap::new(),
            home_articles: Vec::new(),
            link_graph: HashMap::new(),
            slug_to_node_id: HashMap::new(),
            node_id_to_slug: HashMap::new(),
            next_node_id: 0,
            content_cache: HashMap::new(),
        }
    }

    // Load data from ArticlesData and LinkGraphData
    pub fn load_from_data(&mut self, articles_data: ArticlesData, link_graph_data: LinkGraphData) {
        // Clear existing data
        self.articles.clear();
        self.lightweight_articles.clear();
        self.home_articles.clear();
        self.link_graph.clear();
        self.slug_to_node_id.clear();
        self.node_id_to_slug.clear();
        self.next_node_id = 0;
        self.content_cache.clear();

        // Load articles and create lightweight versions
        for article in articles_data.articles {
            let lightweight = LightweightArticle::from(article.clone());
            self.lightweight_articles.insert(article.slug.clone(), lightweight);
            
            // Only keep full articles for home articles in memory initially
            if articles_data.home_articles.contains(&article.slug) {
                self.articles.insert(article.slug.clone(), article);
            }
        }
        
        self.home_articles = articles_data.home_articles;

        // Load link graph
        for (slug, graph_node) in link_graph_data.graph {
            let connections: Vec<String> = graph_node.connections
                .into_iter()
                .map(|conn| conn.target)
                .collect();
            self.link_graph.insert(slug, connections);
        }

        // Assign node IDs to home articles (start from 1 since 0 is reserved for author)
        self.next_node_id = 1; // Reserve 0 for author node
        for slug in &self.home_articles {
            if self.lightweight_articles.contains_key(slug) {
                let node_id = NodeId(self.next_node_id);
                self.slug_to_node_id.insert(slug.clone(), node_id);
                self.node_id_to_slug.insert(node_id, slug.clone());
                self.next_node_id += 1;
            }
        }
    }

    // Load lightweight data only (for initial page load)
    pub fn load_lightweight_data(&mut self, lightweight_articles: Vec<LightweightArticle>, link_graph_data: LinkGraphData) {
        // Clear existing data
        self.articles.clear();
        self.lightweight_articles.clear();
        self.home_articles.clear();
        self.link_graph.clear();
        self.slug_to_node_id.clear();
        self.node_id_to_slug.clear();
        self.next_node_id = 0;
        self.content_cache.clear();

        // Load lightweight articles
        for article in lightweight_articles {
            if article.metadata.home_display {
                self.home_articles.push(article.slug.clone());
            }
            self.lightweight_articles.insert(article.slug.clone(), article);
        }

        // Load link graph
        for (slug, graph_node) in link_graph_data.graph {
            let connections: Vec<String> = graph_node.connections
                .into_iter()
                .map(|conn| conn.target)
                .collect();
            self.link_graph.insert(slug, connections);
        }

        // Assign node IDs to home articles (start from 1 since 0 is reserved for author)
        self.next_node_id = 1; // Reserve 0 for author node
        for slug in &self.home_articles {
            if self.lightweight_articles.contains_key(slug) {
                let node_id = NodeId(self.next_node_id);
                self.slug_to_node_id.insert(slug.clone(), node_id);
                self.node_id_to_slug.insert(node_id, slug.clone());
                self.next_node_id += 1;
            }
        }
    }

    // Get home articles (articles that should be displayed as nodes)
    pub fn get_home_articles(&self) -> Vec<&ProcessedArticle> {
        self.home_articles
            .iter()
            .filter_map(|slug| self.articles.get(slug))
            .collect()
    }

    // Get related articles for a given article slug
    pub fn get_related_articles(&self, slug: &str) -> Vec<&ProcessedArticle> {
        let mut related = Vec::new();

        // Get directly linked articles from link graph
        if let Some(connections) = self.link_graph.get(slug) {
            for connected_slug in connections {
                if let Some(article) = self.articles.get(connected_slug) {
                    related.push(article);
                }
            }
        }

        // Also include articles from metadata related_articles
        if let Some(article) = self.articles.get(slug) {
            for related_slug in &article.metadata.related_articles {
                if let Some(related_article) = self.articles.get(related_slug) {
                    if !related.iter().any(|a| a.slug == related_article.slug) {
                        related.push(related_article);
                    }
                }
            }
        }

        related
    }

    // Get article by slug (returns full article if cached, otherwise None)
    pub fn get_article(&self, slug: &str) -> Option<&ProcessedArticle> {
        self.articles.get(slug)
    }

    // Get lightweight article by slug
    pub fn get_lightweight_article(&self, slug: &str) -> Option<&LightweightArticle> {
        self.lightweight_articles.get(slug)
    }

    // Check if full article content is loaded
    pub fn is_article_loaded(&self, slug: &str) -> bool {
        self.articles.contains_key(slug)
    }

    // Cache full article content
    pub fn cache_article(&mut self, article: ProcessedArticle) {
        self.articles.insert(article.slug.clone(), article);
    }

    // Get cached content or None if not cached
    pub fn get_cached_content(&self, slug: &str) -> Option<&String> {
        self.content_cache.get(slug)
    }

    // Cache content separately (for memory efficiency)
    pub fn cache_content(&mut self, slug: String, content: String) {
        self.content_cache.insert(slug, content);
    }

    // Get all articles (only loaded ones)
    pub fn get_all_articles(&self) -> Vec<&ProcessedArticle> {
        self.articles.values().collect()
    }

    // Get all lightweight articles
    pub fn get_all_lightweight_articles(&self) -> Vec<&LightweightArticle> {
        self.lightweight_articles.values().collect()
    }

    // Get node ID for a given article slug
    pub fn get_node_id(&self, slug: &str) -> Option<NodeId> {
        self.slug_to_node_id.get(slug).copied()
    }

    // Get article slug for a given node ID
    pub fn get_article_slug(&self, node_id: NodeId) -> Option<&String> {
        self.node_id_to_slug.get(&node_id)
    }

    // Check if an article should be displayed on home screen
    pub fn is_home_article(&self, slug: &str) -> bool {
        self.home_articles.contains(&slug.to_string())
    }

    // Get connections for a given article slug
    pub fn get_connections(&self, slug: &str) -> Vec<String> {
        self.link_graph.get(slug).cloned().unwrap_or_default()
    }

    // Create NodeRegistry from current article data
    pub fn create_node_registry(&self) -> NodeRegistry {
        let mut registry = NodeRegistry::new();

        // Add author node at the center (NodeId 0 is reserved for author)
        let center_pos = Position { x: 400.0, y: 300.0 };
        registry.add_author_node(
            center_pos,
            "Khimoo".to_string(), // TODO: Make this configurable
            "/assets/profile.jpg".to_string(), // TODO: Make this configurable
            Some("Software Developer & Tech Enthusiast".to_string()) // TODO: Make this configurable
        );

        // Add nodes for home articles (start from NodeId 1 since 0 is author)
        for (i, slug) in self.home_articles.iter().enumerate() {
            if let Some(article) = self.lightweight_articles.get(slug) {
                let node_id = NodeId((i + 1) as u32); // +1 to skip author node ID
                
                // Calculate node size dynamically based on importance and inbound links
                let radius = registry.calculate_dynamic_radius(
                    node_id, 
                    article.metadata.importance, 
                    article.inbound_count
                );

                // Create node content
                let content = NodeContent::Text(article.title.clone());

                // Position nodes in a circle around the author node, but group by category
                let category = article.metadata.category.clone().unwrap_or_else(|| "default".to_string());
                let category_offset = self.get_category_angle_offset(&category);
                let angle = (i as f32) * 2.0 * std::f32::consts::PI / (self.home_articles.len() as f32) + category_offset;
                let distance = 250.0; // Distance from author node
                let pos = Position {
                    x: center_pos.x + distance * angle.cos(),
                    y: center_pos.y + distance * angle.sin(),
                };

                registry.add_node(node_id, pos, radius, content);
                
                // Set category for the node
                registry.set_node_category(node_id, category);
                
                // Note: slug to node ID mapping is handled in load_from_data method
            }
        }

        // Add edges from author to all article nodes (author as central hub)
        for (i, _slug) in self.home_articles.iter().enumerate() {
            let article_node_id = NodeId((i + 1) as u32);
            registry.add_edge(AUTHOR_NODE_ID, article_node_id);
            
            // Add connection line for author to article
            registry.add_connection_line(
                AUTHOR_NODE_ID, 
                article_node_id, 
                ConnectionLineType::AuthorToArticle, 
                2000.0
            );
        }

        // Add edges based on link graph (between article nodes)
        for (from_slug, connections) in &self.link_graph {
            if let Some(from_node_id) = self.slug_to_node_id.get(from_slug) {
                for to_slug in connections {
                    if let Some(to_node_id) = self.slug_to_node_id.get(to_slug) {
                        // Only add edge if both nodes are in home articles
                        if self.is_home_article(from_slug) && self.is_home_article(to_slug) {
                            registry.add_edge(*from_node_id, *to_node_id);
                            
                            // Check if this is a bidirectional link
                            let is_bidirectional = self.link_graph
                                .get(to_slug)
                                .map_or(false, |reverse_connections| reverse_connections.contains(from_slug));
                            
                            let connection_type = if is_bidirectional {
                                ConnectionLineType::Bidirectional
                            } else {
                                ConnectionLineType::DirectLink
                            };
                            
                            let strength = if is_bidirectional { 8000.0 * 1.5 } else { 8000.0 };
                            
                            // Add connection line for direct link
                            registry.add_connection_line(
                                *from_node_id, 
                                *to_node_id, 
                                connection_type, 
                                strength
                            );
                        }
                    }
                }
            }
        }

        registry
    }

    // Helper method to get angle offset for category clustering
    fn get_category_angle_offset(&self, category: &str) -> f32 {
        // Create consistent angle offsets for different categories
        match category {
            "programming" => 0.0,
            "web" => std::f32::consts::PI / 3.0,      // 60 degrees
            "rust" => 2.0 * std::f32::consts::PI / 3.0, // 120 degrees
            "design" => std::f32::consts::PI,         // 180 degrees
            "tutorial" => 4.0 * std::f32::consts::PI / 3.0, // 240 degrees
            _ => 5.0 * std::f32::consts::PI / 3.0,    // 300 degrees for default
        }
    }

    // Get node sizing data for physics system updates
    pub fn get_node_sizing_data(&self) -> HashMap<NodeId, (Option<u8>, usize)> {
        let mut sizing_data = HashMap::new();
        
        // Add author node (fixed size)
        sizing_data.insert(AUTHOR_NODE_ID, (Some(5), 0)); // Max importance, no inbound links
        
        // Add article nodes
        for (slug, node_id) in &self.slug_to_node_id {
            if let Some(article) = self.lightweight_articles.get(slug) {
                sizing_data.insert(*node_id, (article.metadata.importance, article.inbound_count));
            }
        }
        
        sizing_data
    }

    // Update node sizes based on current data (for dynamic updates)
    pub fn update_node_sizes(&self, registry: &mut NodeRegistry) {
        // Update author node (always fixed size)
        registry.update_node_radius(AUTHOR_NODE_ID, 60);
        
        // Update article nodes
        for (slug, node_id) in &self.slug_to_node_id {
            if let Some(article) = self.lightweight_articles.get(slug) {
                let new_radius = registry.calculate_dynamic_radius(
                    *node_id,
                    article.metadata.importance,
                    article.inbound_count
                );
                registry.update_node_radius(*node_id, new_radius);
            }
        }
    }

    // Get lightweight articles by category
    pub fn get_lightweight_articles_by_category(&self, category: &str) -> Vec<&LightweightArticle> {
        self.lightweight_articles
            .values()
            .filter(|article| {
                article.metadata.category.as_ref().map_or(false, |cat| cat == category)
            })
            .collect()
    }

    // Get articles by category (full articles only)
    pub fn get_articles_by_category(&self, category: &str) -> Vec<&ProcessedArticle> {
        self.articles
            .values()
            .filter(|article| {
                article.metadata.category.as_ref().map_or(false, |cat| cat == category)
            })
            .collect()
    }

    // Get all categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.lightweight_articles
            .values()
            .filter_map(|article| article.metadata.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }

    // Get lightweight articles by tag
    pub fn get_lightweight_articles_by_tag(&self, tag: &str) -> Vec<&LightweightArticle> {
        self.lightweight_articles
            .values()
            .filter(|article| article.metadata.tags.contains(&tag.to_string()))
            .collect()
    }

    // Get articles by tag (full articles only)
    pub fn get_articles_by_tag(&self, tag: &str) -> Vec<&ProcessedArticle> {
        self.articles
            .values()
            .filter(|article| article.metadata.tags.contains(&tag.to_string()))
            .collect()
    }

    // Get all tags
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.lightweight_articles
            .values()
            .flat_map(|article| article.metadata.tags.iter().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        tags.sort();
        tags
    }

    // Search lightweight articles by title or summary
    pub fn search_lightweight_articles(&self, query: &str) -> Vec<&LightweightArticle> {
        let query_lower = query.to_lowercase();
        self.lightweight_articles
            .values()
            .filter(|article| {
                article.title.to_lowercase().contains(&query_lower) ||
                article.summary.as_ref().map_or(false, |summary| summary.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    // Search articles by title or content (full articles only)
    pub fn search_articles(&self, query: &str) -> Vec<&ProcessedArticle> {
        let query_lower = query.to_lowercase();
        self.articles
            .values()
            .filter(|article| {
                article.title.to_lowercase().contains(&query_lower) ||
                article.content.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    // Get statistics
    pub fn get_stats(&self) -> ArticleStats {
        let total_articles = self.lightweight_articles.len();
        let home_articles_count = self.home_articles.len();
        let total_connections = self.link_graph.values().map(|v| v.len()).sum();
        let categories_count = self.get_categories().len();
        let tags_count = self.get_all_tags().len();

        ArticleStats {
            total_articles,
            home_articles_count,
            total_connections,
            categories_count,
            tags_count,
        }
    }
}

impl Default for ArticleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArticleStats {
    pub total_articles: usize,
    pub home_articles_count: usize,
    pub total_connections: usize,
    pub categories_count: usize,
    pub tags_count: usize,
}

// Hook for using ArticleManager with data loading
#[hook]
pub fn use_article_manager() -> (
    UseStateHandle<Option<ArticleManager>>,
    UseStateHandle<bool>,
    UseStateHandle<Option<DataLoadError>>
) {
    let manager = use_state(|| None);
    let loading = use_state(|| true);
    let error = use_state(|| None);

    {
        let manager = manager.clone();
        let loading = loading.clone();
        let error = error.clone();
        
        use_effect_with((), move |_| {
            let manager = manager.clone();
            let loading = loading.clone();
            let error = error.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                let loader = super::data_loader::DataLoader::new();
                let (articles_result, link_graph_result) = loader.load_all_data().await;
                
                match (articles_result, link_graph_result) {
                    (Ok(articles_data), Ok(link_graph_data)) => {
                        let mut article_manager = ArticleManager::new();
                        article_manager.load_from_data(articles_data, link_graph_data);
                        manager.set(Some(article_manager));
                        error.set(None);
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        error.set(Some(e));
                    }
                }
                loading.set(false);
            });
            
            || {}
        });
    }

    (manager, loading, error)
}

// Hook for using ArticleManager with lightweight data loading (faster initial load)
#[hook]
pub fn use_lightweight_article_manager() -> (
    UseStateHandle<Option<ArticleManager>>,
    UseStateHandle<bool>,
    UseStateHandle<Option<DataLoadError>>
) {
    let manager = use_state(|| None);
    let loading = use_state(|| true);
    let error = use_state(|| None);

    {
        let manager = manager.clone();
        let loading = loading.clone();
        let error = error.clone();
        
        use_effect_with((), move |_| {
            let manager = manager.clone();
            let loading = loading.clone();
            let error = error.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                let loader = DataLoader::new();
                let lightweight_future = loader.load_lightweight_articles();
                let link_graph_future = loader.load_link_graph();
                
                let (lightweight_result, link_graph_result) = futures::join!(lightweight_future, link_graph_future);
                
                match (lightweight_result, link_graph_result) {
                    (Ok(lightweight_articles), Ok(link_graph_data)) => {
                        let mut article_manager = ArticleManager::new();
                        article_manager.load_lightweight_data(lightweight_articles, link_graph_data);
                        manager.set(Some(article_manager));
                        error.set(None);
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        error.set(Some(e));
                    }
                }
                loading.set(false);
            });
            
            || {}
        });
    }

    (manager, loading, error)
}

// Hook for lazy loading a specific article into the manager
#[hook]
pub fn use_lazy_article_loader(
    manager: UseStateHandle<Option<ArticleManager>>,
    slug: Option<String>
) -> (UseStateHandle<bool>, UseStateHandle<Option<DataLoadError>>) {
    let loading = use_state(|| false);
    let error = use_state(|| None);

    {
        let manager = manager.clone();
        let loading = loading.clone();
        let error = error.clone();
        
        use_effect_with(slug.clone(), move |slug| {
            if let (Some(slug), Some(current_manager)) = (slug, (*manager).clone()) {
                // Check if article is already loaded
                if !current_manager.is_article_loaded(slug) {
                    let manager = manager.clone();
                    let loading = loading.clone();
                    let error = error.clone();
                    let slug = slug.clone();
                    
                    loading.set(true);
                    error.set(None);
                    
                    wasm_bindgen_futures::spawn_local(async move {
                        let loader = DataLoader::new();
                        match loader.load_article_by_slug(&slug).await {
                            Ok(article) => {
                                if let Some(mut current_manager) = (*manager).clone() {
                                    current_manager.cache_article(article);
                                    manager.set(Some(current_manager));
                                }
                                error.set(None);
                            }
                            Err(e) => {
                                error.set(Some(e));
                            }
                        }
                        loading.set(false);
                    });
                }
            }
            
            || {}
        });
    }

    (loading, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::data_loader::ProcessedMetadata;

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
}