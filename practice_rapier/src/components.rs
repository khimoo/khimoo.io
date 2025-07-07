use crate::physics_sim::PhysicsWorld;
use crate::types::{ContainerBound, Node, NodeId, NodePosition, Position};
use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::{
    function_component, html, use_state, Callback, Html, MouseEvent, NodeRef, Properties,
};
use yew_hooks::{use_interval, UseMeasureState};

#[derive(Properties, PartialEq)]
pub struct NodeGraphContainerProps {
    pub container_ref: NodeRef,
    pub container_measure: UseMeasureState,
    pub container_bound: ContainerBound,
    pub window_mouse_pos: Position,
    pub global_mouse_pos: Position,
    pub window_scroll: Position,
}

#[function_component(NodeGraphContainer)]
pub fn node_graph_container(props: &NodeGraphContainerProps) -> Html {
    let dragged_node_id = use_state(|| None::<NodeId>);

    let initial_nodes = vec![
        Node {
            id: 0,
            pos: NodePosition { x: 100, y: 150 },
            radius: 30,
        },
        Node {
            id: 1,
            pos: NodePosition { x: 300, y: 250 },
            radius: 50,
        },
    ];
    let nodes_handle = use_state(|| initial_nodes.clone());

    let physics_world = use_state(|| Rc::new(RefCell::new(PhysicsWorld::new(&initial_nodes))));

    let on_mouse_move = {
        let global_mouse_pos = props.global_mouse_pos.clone();
        let dragged_node_id = dragged_node_id.clone();
        let physics_world = physics_world.clone();
        Callback::from(move |_| {
            if let Some(id) = *dragged_node_id {
                let mut world = physics_world.borrow_mut();
                world.set_node_position(
                    id,
                    &NodePosition {
                        x: global_mouse_pos.x as i32,
                        y: global_mouse_pos.y as i32,
                    },
                );
            }
        })
    };

    let on_mouse_down = {
        let dragged_node_id = dragged_node_id.clone();
        let physics_world = physics_world.clone();
        Callback::from(move |id: NodeId| {
            // physics_world.borrow_mut().set_node_kinematic(id);
            dragged_node_id.set(Some(id));
        })
    };

    let on_mouse_up = {
        let dragged_node_id = dragged_node_id.clone();
        let physics_world = physics_world.clone();
        Callback::from(move |_: MouseEvent| {
            // if let Some(id) = *dragged_node_id {
            //     physics_world.borrow_mut().set_node_dynamic(id);
            // }
            dragged_node_id.set(None);
        })
    };

    let physics_zero = use_state(|| NodePosition::default());

    {
        let nodes_handle = nodes_handle.clone();
        let physics_world = physics_world.clone();
        let physics_zero = physics_zero.clone();
        let dragged_node_id = dragged_node_id.clone();
        use_interval(
            move || {
                let mut world = physics_world.borrow_mut();
                if dragged_node_id.is_none() {
                    world.step();
                }
                nodes_handle.set(world.get_nodes());
                physics_zero.set(world.get_zero());
            },
            16,
        ); // ~60fps
    }
    html! {
        <>
                <div
                    style="position: static; width: 100vw; height: 100vh; background: #f0f0f0;"
                    onmousemove={on_mouse_move} onmouseup={on_mouse_up}
                    ref={props.container_ref.clone()}
                >
                    <h1>{"node_graph"}</h1>
                    <p>{ format!("global_mouse_pos({},{})", props.global_mouse_pos.x,props.global_mouse_pos.y)}</p>
                    <p>{ format!("{:?}", props.container_bound)}</p>
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
                                         width: {}px;
                                         height: {}px;
                                         background-color: black;
                                         border-radius: 50%;
                                         transform: translate(-50%, -50%);
                                         left: {}px;
                                         top: {}px;
                                         box-shadow: 0 4px 8px rgba(0,0,0,0.2);
                                         z-index: 10;",
                                        node.radius, node.radius, node.pos.x, node.pos.y
                                    )}
                                ></div>
                            }
                        }).collect::<Html>()
                    }
                    <div style={
                        format!("position: absolute;
                        left: {}px;
                        top: {}px;
                        background-color: black;
                        transform: translate(-50%, -50%);
                        width: 10px;
                        height: 10px;
                        border-radius: 50%;", physics_zero.x, physics_zero.y)}></div>
                </div>
        </>
    }
}

#[function_component(Interval)]
pub fn interval() -> Html {
    let state = use_state(|| 0);

    {
        let state = state.clone();
        use_interval(
            move || {
                state.set(*state + 1);
            },
            2000,
        );
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
