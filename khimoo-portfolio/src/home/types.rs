use std::collections::HashMap;
use yew::{html, Html};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ForceSettings {
    pub repulsion_strength: f32,
    pub repulsion_min_distance: f32,
    pub link_strength: f32,
    pub center_strength: f32,
    pub center_damping: f32,
    // Author node specific settings
    pub author_attraction_strength: f32,
    pub author_attraction_damping: f32,
    pub author_fixed_position: bool,
    // Direct link specific settings
    pub direct_link_strength: f32,
    pub direct_link_damping: f32,
    pub bidirectional_link_multiplier: f32,
    // Debug mode settings
    pub debug_mode: bool,
    pub show_connection_lines: bool,
    pub connection_line_opacity: f32,
    // Category-based clustering settings
    pub category_attraction_strength: f32,
    pub category_attraction_range: f32,
    pub enable_category_clustering: bool,
}

impl Default for ForceSettings {
    fn default() -> Self {
        Self {
            repulsion_strength: 50000.0,
            repulsion_min_distance: 400.0,
            link_strength: 5000.0,
            center_strength: 3000.0,
            center_damping: 5.0,
            // Author node defaults
            author_attraction_strength: 2000.0,
            author_attraction_damping: 8.0,
            author_fixed_position: true,
            // Direct link defaults
            direct_link_strength: 8000.0,
            direct_link_damping: 300.0,
            bidirectional_link_multiplier: 1.5,
            // Debug mode defaults
            debug_mode: false,
            show_connection_lines: true,
            connection_line_opacity: 0.6,
            // Category clustering defaults
            category_attraction_strength: 1500.0,
            category_attraction_range: 300.0,
            enable_category_clustering: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ContainerBound {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct NodeId(pub u32);

// Special node ID for the author node (always 0)
pub const AUTHOR_NODE_ID: NodeId = NodeId(0);

#[derive(Clone, PartialEq)]
pub enum NodeType {
    Author,
    Article,
}

#[derive(Clone, PartialEq)]
pub enum NodeContent {
    Text(String),
    Image(String), // 画像URLのみ
    Link { text: String, url: String },
    Author { name: String, image_url: String, bio: Option<String> },
}

impl Default for NodeContent {
    fn default() -> Self {
        NodeContent::Text("".to_string())
    }
}

impl NodeContent {
    pub fn render_content(&self) -> Html {
        match self {
            NodeContent::Text(text) => html! {
                <span style="color: white; font-size: 12px;">
                    {text}
                </span>
            },
            NodeContent::Image(url) => html! {
                <img
                    src={url.clone()}
                    style="max-width: 100%; max-height: 100%; object-fit: contain;"
                />
            },
            NodeContent::Link { text, url } => html! {
                <a
                    href={url.clone()}
                    style="color: lightblue; text-decoration: none; font-size: 12px;"
                >
                    {text}
                </a>
            },
            NodeContent::Author { name, image_url, bio } => html! {
                <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; width: 100%;">
                    <img
                        src={image_url.clone()}
                        style="width: 80%; height: 80%; border-radius: 50%; object-fit: cover; border: 3px solid #fff;"
                        alt={format!("Profile picture of {}", name)}
                    />
                    <div style="color: white; font-size: 14px; font-weight: bold; margin-top: 5px; text-align: center;">
                        {name}
                    </div>
                    if let Some(bio_text) = bio {
                        <div style="color: #ccc; font-size: 10px; text-align: center; margin-top: 2px; max-width: 90%; overflow: hidden; text-overflow: ellipsis;">
                            {bio_text}
                        </div>
                    }
                </div>
            },
        }
    }

    pub fn get_node_type(&self) -> NodeType {
        match self {
            NodeContent::Author { .. } => NodeType::Author,
            _ => NodeType::Article,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionLine {
    pub from: NodeId,
    pub to: NodeId,
    pub connection_type: ConnectionLineType,
    pub strength: f32,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionLineType {
    DirectLink,
    Bidirectional,
    AuthorToArticle,
}

pub struct NodeRegistry {
    pub positions: HashMap<NodeId, Position>,
    pub radii: HashMap<NodeId, i32>,
    pub contents: HashMap<NodeId, NodeContent>,
    pub edges: Vec<(NodeId, NodeId)>,
    pub node_types: HashMap<NodeId, NodeType>,
    pub connection_lines: Vec<ConnectionLine>,
    pub node_categories: HashMap<NodeId, String>,
    pub category_colors: HashMap<String, CategoryColor>,
    pub node_importance: HashMap<NodeId, u8>,
    pub node_inbound_counts: HashMap<NodeId, usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryColor {
    pub primary: String,   // Main node color
    pub secondary: String, // Border or accent color
    pub text: String,      // Text color for contrast
}

impl NodeRegistry {
    pub fn new() -> Self {
        let mut category_colors = HashMap::new();
        
        // Default category colors
        category_colors.insert("programming".to_string(), CategoryColor {
            primary: "#4A90E2".to_string(),   // Blue
            secondary: "#357ABD".to_string(),
            text: "#FFFFFF".to_string(),
        });
        category_colors.insert("web".to_string(), CategoryColor {
            primary: "#7ED321".to_string(),   // Green
            secondary: "#5BA517".to_string(),
            text: "#FFFFFF".to_string(),
        });
        category_colors.insert("rust".to_string(), CategoryColor {
            primary: "#CE422B".to_string(),   // Rust orange
            secondary: "#A0341F".to_string(),
            text: "#FFFFFF".to_string(),
        });
        category_colors.insert("design".to_string(), CategoryColor {
            primary: "#BD10E0".to_string(),   // Purple
            secondary: "#9013B0".to_string(),
            text: "#FFFFFF".to_string(),
        });
        category_colors.insert("tutorial".to_string(), CategoryColor {
            primary: "#F5A623".to_string(),   // Orange
            secondary: "#D1891C".to_string(),
            text: "#FFFFFF".to_string(),
        });
        category_colors.insert("default".to_string(), CategoryColor {
            primary: "#9B9B9B".to_string(),   // Gray
            secondary: "#7B7B7B".to_string(),
            text: "#FFFFFF".to_string(),
        });

        Self {
            positions: HashMap::new(),
            radii: HashMap::new(),
            contents: HashMap::new(),
            edges: Vec::new(),
            node_types: HashMap::new(),
            connection_lines: Vec::new(),
            node_categories: HashMap::new(),
            category_colors,
            node_importance: HashMap::new(),
            node_inbound_counts: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: NodeId, pos: Position, radius: i32, content: NodeContent) {
        let node_type = content.get_node_type();
        self.positions.insert(id, pos);
        self.radii.insert(id, radius);
        self.contents.insert(id, content);
        self.node_types.insert(id, node_type);
    }

    pub fn add_author_node(&mut self, pos: Position, name: String, image_url: String, bio: Option<String>) {
        let content = NodeContent::Author { name, image_url, bio };
        // Author node gets a larger radius
        let radius = 60; // Larger than regular article nodes
        self.add_node(AUTHOR_NODE_ID, pos, radius, content);
    }

    pub fn get_node_type(&self, id: NodeId) -> Option<&NodeType> {
        self.node_types.get(&id)
    }

    pub fn is_author_node(&self, id: NodeId) -> bool {
        matches!(self.get_node_type(id), Some(NodeType::Author))
    }

    pub fn get_author_node_id(&self) -> Option<NodeId> {
        self.node_types.iter()
            .find(|(_, node_type)| matches!(node_type, NodeType::Author))
            .map(|(id, _)| *id)
    }

    pub fn add_connection_line(&mut self, from: NodeId, to: NodeId, connection_type: ConnectionLineType, strength: f32) {
        let line = ConnectionLine {
            from,
            to,
            connection_type,
            strength,
            visible: true,
        };
        self.connection_lines.push(line);
    }

    pub fn get_connection_lines(&self) -> &Vec<ConnectionLine> {
        &self.connection_lines
    }

    pub fn set_connection_line_visibility(&mut self, visible: bool) {
        for line in &mut self.connection_lines {
            line.visible = visible;
        }
    }

    pub fn set_node_category(&mut self, node_id: NodeId, category: String) {
        self.node_categories.insert(node_id, category);
    }

    pub fn get_node_category(&self, node_id: NodeId) -> Option<&String> {
        self.node_categories.get(&node_id)
    }

    pub fn get_category_color(&self, category: &str) -> &CategoryColor {
        self.category_colors.get(category).unwrap_or_else(|| {
            self.category_colors.get("default").unwrap()
        })
    }

    pub fn get_nodes_by_category(&self, category: &str) -> Vec<NodeId> {
        self.node_categories
            .iter()
            .filter(|(_, cat)| *cat == category)
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn get_all_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.node_categories
            .values()
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }

    pub fn update_node_radius(&mut self, node_id: NodeId, new_radius: i32) {
        self.radii.insert(node_id, new_radius);
    }

    pub fn calculate_dynamic_radius(&self, node_id: NodeId, importance: Option<u8>, inbound_count: usize) -> i32 {
        let base_size = if self.is_author_node(node_id) {
            60 // Author node is always larger
        } else {
            30 // Base size for article nodes
        };

        if self.is_author_node(node_id) {
            return base_size; // Author node size is fixed
        }

        // Calculate size based on importance (1-5 scale)
        let importance_multiplier = importance.unwrap_or(3) as i32;
        let importance_bonus = (importance_multiplier - 3) * 8; // -16 to +16

        // Calculate size based on inbound links (popularity)
        let inbound_multiplier = (inbound_count as f32).sqrt() as i32;
        let inbound_bonus = inbound_multiplier * 4; // 0 to ~20 for typical link counts

        // Ensure minimum and maximum sizes
        let calculated_size = base_size + importance_bonus + inbound_bonus;
        calculated_size.clamp(20, 80) // Min 20px, Max 80px
    }

    pub fn iter(&self) -> impl Iterator<Item = (&NodeId, &Position, &i32, &NodeContent)> {
        self.positions.iter().filter_map(move |(id, pos)| {
            let radius = self.radii.get(id)?;
            let content = self.contents.get(id)?;
            Some((id, pos, radius, content))
        })
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.edges.push((from, to));
    }

    pub fn iter_edges(&self) -> impl Iterator<Item = &(NodeId, NodeId)> {
        self.edges.iter()
    }

    pub fn set_node_importance(&mut self, node_id: NodeId, importance: u8) {
        self.node_importance.insert(node_id, importance);
    }

    pub fn get_node_importance(&self, node_id: NodeId) -> Option<u8> {
        self.node_importance.get(&node_id).copied()
    }

    pub fn set_node_inbound_count(&mut self, node_id: NodeId, count: usize) {
        self.node_inbound_counts.insert(node_id, count);
    }

    pub fn get_node_inbound_count(&self, node_id: NodeId) -> usize {
        self.node_inbound_counts.get(&node_id).copied().unwrap_or(0)
    }
}
