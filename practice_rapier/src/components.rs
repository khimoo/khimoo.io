use yew::prelude::{ function_component, Properties, Html, html, NodeRef, use_state, Callback, MouseEvent };
use crate::types::{MousePosition, ContainerMeasure, Nodes, Node, NodePosition};
use crate::physics_sim::{ PhysicsWorld };
use yew_hooks::use_interval;
use std::rc::Rc;
use std::cell::RefCell;


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

    let initial_nodes = vec![
        Node { id: 0, pos: NodePosition { x: 100, y: 150 } },
        Node { id: 1, pos: NodePosition { x: 300, y: 250 } },
    ];
    let nodes_handle = use_state(|| initial_nodes.clone());

    let physics_world = use_state(|| {
        Rc::new(RefCell::new(PhysicsWorld::new(&initial_nodes)))
    });

    {
        let nodes_handle = nodes_handle.clone();
        let physics_world = physics_world.clone();
        use_interval(move || {
            let mut world = physics_world.borrow_mut();
            world.step();
            nodes_handle.set(world.get_nodes());
        }, 16); // ~60fps
    }
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
                        nodes_handle.iter().map(|node| html! {
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
