use yew::{html, Html};
use yew::prelude::{Callback, MouseEvent};
use yew::virtual_dom::VNode;

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
pub struct NodeBase {
    pub id: NodeId,
    pub pos: Position,
    pub radius: i32,

}

#[derive(Clone, PartialEq)]
pub struct Node {
    pub base: NodeBase,
    pub content: NodeContent,
}

#[derive(Clone, PartialEq)]
pub enum NodeContent {
    Text(String),
    Image(String), // 画像URLのみ
    Link { text: String, url: String },
}

impl Node {
    pub fn get_div(&self, on_mouse_down: Callback<MouseEvent>) -> VNode {
        html! {
            <div
                key={self.base.id.to_string()}
                onmousedown={on_mouse_down}
                style={format!(
                    "position: absolute;
                    width: {}px;
                    height: {}px;
                    background-color: black;
                    border-radius: 50%;
                    transform: translate(-50%, -50%);
                    left: {}px;
                    top: {}px;
                    box-shadow: 0 4px 8px rgba(0,0,0,0.2);
                    z-index: 10;
                    display: flex;
                    justify-content: center;
                    align-items: center;",
                    2 * self.base.radius,
                    2 * self.base.radius,
                    self.base.pos.x,
                    self.base.pos.y
                )}
            >
                <div style="max-width: 80%; max-height: 80%; overflow: hidden;">
                    {self.render_content()}
                </div>
            </div>
        }
    }

    // コンテンツのレンダリング（シンプル版）
    fn render_content(&self) -> Html {
        match &self.content {
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
        }
    }
}

pub type Nodes = Vec<Node>;
