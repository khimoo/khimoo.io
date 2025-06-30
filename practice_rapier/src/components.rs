use yew::prelude::{ function_component, Properties, Html, html, NodeRef };
use crate::types::{MousePosition, ContainerMeasure};


#[derive(Properties, PartialEq)]
pub struct NodeGraphContainerProps {
    pub mouse_position: MousePosition,
    pub container_ref: NodeRef,
    pub container_measure: ContainerMeasure,
}

#[function_component(NodeGraphContainer)]
pub fn node_graph_container(props: &NodeGraphContainerProps) -> Html {
    html! {
        <>
            <h1>{"node_graph"}</h1>
            <p>{ format!("{}", props.mouse_position.x)}</p>
            <p>{ format!("{}", props.container_measure.width)}</p>
            <div
                style="width: 100vw; height: 100vh; background: #f0f0f0;"
                ref={props.container_ref.clone()}
            >
            </div>
        </>
    }
}
