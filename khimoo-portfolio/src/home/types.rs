use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ContainerBound {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub top: i32,
    pub left: i32,
    pub bottom: i32,
    pub right: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct NodeId(pub u32);

#[derive(Clone, PartialEq)]
pub enum NodeContent {
    Text(String),
    Image(String), // 画像URLのみ
    Link { text: String, url: String },
}

impl Default for NodeContent {
    fn default() -> Self {
        NodeContent::Text("".to_string())
    }
}

pub struct NodeRegistry {
    pub positions: HashMap<NodeId, Position>,
    pub radii: HashMap<NodeId, i32>,
    pub contents: HashMap<NodeId, NodeContent>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            radii: HashMap::new(),
            contents: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: NodeId, pos: Position, radius: i32, content: NodeContent) {
        self.positions.insert(id, pos);
        self.radii.insert(id, radius);
        self.contents.insert(id, content);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&NodeId, &Position, &i32, &NodeContent)> {
        self.positions.iter().filter_map(move |(id, pos)| {
            let radius = self.radii.get(id)?;
            let content = self.contents.get(id)?;
            Some((id, pos, radius, content))
        })
    }
}
