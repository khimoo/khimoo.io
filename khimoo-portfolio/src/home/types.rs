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

pub type NodeId = u32;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct NodePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub pos: NodePosition,
    pub radius: i32,
}

pub type Nodes = Vec<Node>;
