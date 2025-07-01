#[derive(PartialEq, Copy, Clone)]
pub struct MousePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Default, Clone, PartialEq)]
pub struct ContainerMeasure {
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

#[derive(Clone)]
pub struct NodePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone)]
pub struct Node {
    pub id: NodeId,
    pub pos: NodePosition,
}

pub type Nodes = Vec<Node>;

