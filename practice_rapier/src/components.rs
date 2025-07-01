use yew::prelude::{ function_component, Properties, Html, html, NodeRef, use_state, Callback, MouseEvent };
use crate::types::{MousePosition, ContainerMeasure, Nodes};
use crate::physics_sim::{ PhysicsStepResources };
use yew_hooks::use_interval;


#[derive(Properties, PartialEq)]
pub struct NodeGraphContainerProps {
    pub container_ref: NodeRef,
    pub container_measure: ContainerMeasure,
}

#[function_component(NodeGraphContainer)]
pub fn node_graph_container(props: &NodeGraphContainerProps) -> Html {
    let mouse_position_handle = use_state(|| MousePosition{x:0,y:0});
    // マウスが動いたときのイベントハンドラ
    let on_mouse_move = {
        let mouse_position_handle = mouse_position_handle.clone();
        Callback::from(move |e: MouseEvent| {
            // ビューポート（ウィンドウ）基準の座標を取得
            mouse_position_handle.set(MousePosition { x: e.client_x(), y: e.client_y() });
        })
    };

    let mut nodes = Nodes::new();
    let nodes_handle = use_state(|| nodes.clone());
    // let physics_world_handle = use_state(||

    let physics_world = use_state(|| {
        let world = PhysicsStepResources::new(None, None);
        std::rc::Rc::new(std::cell::RefCell::new(world))
    });
    let nodes = nodes.clone();
    let nodes_handle = nodes_handle.clone();
    use_interval(move || {
        let new_nodes = PhysicsStepResources::physics_step();
        nodes_handle.set(new_nodes);
    }, 100);


    // テスト用ノードを2つ定義
    html! {
        <>
            <div onmousemove={on_mouse_move}>
                <h1>{"node_graph"}</h1>
                <p>{ format!("({},{})", mouse_position_handle.x, mouse_position_handle.y)}</p>
                <p>{ format!("{}", props.container_measure.width)}</p>
                <div
                    style="position: relative; width: 100vw; height: 100vh; background: #f0f0f0;"
                    ref={props.container_ref.clone()}
                >
                    {
                        nodes.iter().map(|node| html! {
                            <div key={node.id.to_string()}
                                style={
                                    format!("position: absolute;
                                     width: 50px;
                                     height: 50px;
                                     background-color: black;
                                     border-radius: 50%;
                                     transform: translate(-50%, -50%);
                                     left: {}px;
                                     top: {}px;
                                     box-shadow: 0 4px 8px rgba(0,0,0,0.2);
                                     z-index: 10;",
                                    node.pos.x, node.pos.y
                                )}
                            ></div>
                        }).collect::<Html>()
                    }
                </div>
            </div>
        </>
    }
}

#[function_component(Interval)]
pub fn interval() -> Html {
    let state = use_state(|| 0);

    {
        let state = state.clone();
        use_interval(move || {
            state.set(*state + 1);
        }, 2000);
    }

    let on_reset = {
        let state = state.clone();
        Callback::from(move |_| state.set(0))
    };

    html! {
        <>
            <p>{ *state }</p>
            <button onclick={on_reset}>{ "リセット" }</button>
        </>
    }
}





//use yew::prelude::{function_component, html, Callback, Html, MouseEvent, NodeRef, Properties, use_state};
//use crate::types::{MousePosition, ContainerMeasure};
//use std::collections::HashMap;
//use std::rc::Rc;
//use std::cell::RefCell;
//
//#[derive(Properties, PartialEq)]
//pub struct NodeGraphContainerProps {
//    pub container_ref: NodeRef,
//    pub container_measure: ContainerMeasure,
//}
//
//type NodeId = u32;
//
//#[derive(Clone)]
//struct NodePosition {
//    x: i32,
//    y: i32,
//}
//
//type Nodes = HashMap<NodeId, NodePosition>;
//
//#[function_component(NodeGraphContainer)]
//pub fn node_graph_container(props: &NodeGraphContainerProps) -> Html {
//    let mouse_position_handle = use_state(|| MousePosition { x: 0, y: 0 });
//    let dragging_node = use_state(|| None::<NodeId>);
//    let nodes = use_state(|| {
//        let mut n = Nodes::new();
//        n.insert(1, NodePosition { x: 100, y: 150 });
//        n.insert(2, NodePosition { x: 300, y: 250 });
//        n
//    });
//
//    let nodes_ref = Rc::new(RefCell::new(nodes.clone()));
//
//    // マウス移動で位置更新
//    let on_mouse_move = {
//        let mouse_position_handle = mouse_position_handle.clone();
//        let nodes = nodes.clone();
//        let dragging_node = dragging_node.clone();
//        Callback::from(move |e: MouseEvent| {
//            let x = e.client_x();
//            let y = e.client_y();
//            mouse_position_handle.set(MousePosition { x, y });
//            if let Some(id) = *dragging_node {
//                let mut new_nodes = (*nodes).clone();
//                if let Some(node) = new_nodes.get_mut(&id) {
//                    node.x = x;
//                    node.y = y;
//                }
//                nodes.set(new_nodes);
//            }
//        })
//    };
//
//    let on_mouse_up = {
//        let dragging_node = dragging_node.clone();
//        Callback::from(move |_| {
//            dragging_node.set(None);
//        })
//    };
//
//    html! {
//        <div
//            onmousemove={on_mouse_move.clone()}
//            onmouseup={on_mouse_up}
//            style="width: 100vw; height: 100vh; position: relative; background: #f0f0f0;"
//            ref={props.container_ref.clone()}
//        >
//            <h1>{"node_graph"}</h1>
//            <p>{ format!("Mouse: ({}, {})", mouse_position_handle.x, mouse_position_handle.y) }</p>
//            <p>{ format!("Container: {}", props.container_measure.width) }</p>
//            {
//                nodes.iter().map(|(id, pos)| {
//                    let dragging_node = dragging_node.clone();
//                    let id = *id;
//                    let on_mouse_down = {
//                        Callback::from(move |_| {
//                            dragging_node.set(Some(id));
//                        })
//                    };
//                    html! {
//                        <div key={id.to_string()}
//                            onmousedown={on_mouse_down}
//                            style={format!"
//                                position: absolute;
//                                width: 50px;
//                                height: 50px;
//                                background-color: black;
//                                border-radius: 50%;
//                                transform: translate(-50%, -50%);
//                                left: {}px;
//                                top: {}px;
//                                box-shadow: 0 4px 8px rgba(0,0,0,0.2);
//                                z-index: 10;
//                            ", pos.x, pos.y)}
//                        ></div>
//                    }
//                }).collect::<Html>()
//            }
//        </div>
//    }
//}
