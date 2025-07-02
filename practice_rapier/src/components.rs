use yew::prelude::{ function_component, Properties, Html, html, NodeRef, use_state, Callback, MouseEvent };
use crate::types::{MousePosition, ContainerMeasure, Nodes, Node, NodePosition, NodeId};
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
    let dragged_node_id = use_state(|| None::<NodeId>);

    let initial_nodes = vec![
        Node { id: 0, pos: NodePosition { x: 100, y: 150 } },
        Node { id: 1, pos: NodePosition { x: 300, y: 250 } },
    ];
    let nodes_handle = use_state(|| initial_nodes.clone());

    let physics_world = use_state(|| {
        Rc::new(RefCell::new(PhysicsWorld::new(&initial_nodes)))
    });

    let on_mouse_move = {
        let mouse_position_handle = mouse_position_handle.clone();
        let dragged_node_id = dragged_node_id.clone();
        let physics_world = physics_world.clone();
        Callback::from(move |e: MouseEvent| {
            let pos = MousePosition { x: e.client_x(), y: e.client_y() };
            mouse_position_handle.set(pos.clone());
            if let Some(id) = *dragged_node_id {
                let mut world = physics_world.borrow_mut();
                world.set_node_position(id, &NodePosition { x: pos.x, y: pos.y });
            }
        })
    };

    let on_mouse_down = {
        let dragged_node_id = dragged_node_id.clone();
        Callback::from(move |id: NodeId| {
            dragged_node_id.set(Some(id));
        })
    };

    let on_mouse_up = {
        let dragged_node_id = dragged_node_id.clone();
        Callback::from(move |_: MouseEvent| {
            dragged_node_id.set(None);
        })
    };


    {
        let nodes_handle = nodes_handle.clone();
        let physics_world = physics_world.clone();
        let dragged_node_id = dragged_node_id.clone();
        use_interval(move || {
            if dragged_node_id.is_none() {
                let mut world = physics_world.borrow_mut();
                world.step();
                nodes_handle.set(world.get_nodes());
            }
        }, 16); // ~60fps
    }
    html! {
        <>
            <div onmousemove={on_mouse_move} onmouseup={on_mouse_up}>
                <h1>{"node_graph"}</h1>
                <p>{ format!("({},{})", mouse_position_handle.x, mouse_position_handle.y)}</p>
                <p>{ format!("{}", props.container_measure.width)}</p>
                <div
                    style="position: relative; width: 100vw; height: 100vh; background: #f0f0f0;"
                    ref={props.container_ref.clone()}
                >
                    {
                        nodes_handle.iter().map(|node| {
                            let on_mouse_down = {
                                let on_mouse_down = on_mouse_down.clone();
                                let id = node.id;
                                Callback::from(move |e: MouseEvent| {
                                    e.stop_propagation();
                                    on_mouse_down.emit(id);
                                })
                            };
                            html! {
                                <div key={node.id.to_string()}
                                    onmousedown={on_mouse_down}
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
                            }
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
