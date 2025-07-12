use super::physics_sim::PhysicsWorld;
use super::types::*;
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
            base: NodeBase {
                id: 0,
                pos: Position { x: 100, y: 150 },
                radius: 30,
            },
            content: NodeContent::Text("node 0".to_string()),
        },
        Node {
            base: NodeBase {
                id: 1,
                pos: Position { x: 200, y: 250 },
                radius: 50,
            },
            content: NodeContent::Text("hello".to_string()),

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
                    &Position {
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
            physics_world.borrow_mut().set_node_kinematic(id);
            dragged_node_id.set(Some(id));
        })
    };

    let on_mouse_up = {
        let dragged_node_id = dragged_node_id.clone();
        let physics_world = physics_world.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(id) = *dragged_node_id {
                physics_world.borrow_mut().set_node_dynamic(id);
            }
            dragged_node_id.set(None);
        })
    };

    let physics_zero = use_state(|| Position::default());

    {
        let nodes_handle = nodes_handle.clone();
        let physics_world = physics_world.clone();
        use_interval(
            move || {
                let mut world = physics_world.borrow_mut();
                world.step();
                let updated_bases = world.get_node_bases();
                let updated_nodes = nodes_handle.iter()
                    .map(|node| {
                        if let Some(new_base) = updated_bases.iter()
                            .find(|base| base.id == node.base.id)
                        {
                            Node {
                                base: *new_base,
                                content: node.content.clone()
                            }
                        } else { node.clone() }
                    }).collect();
                nodes_handle.set(updated_nodes);
                },
            16, ); // ~60fps
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
                                let id = node.base.id;
                                Callback::from(move |e: MouseEvent| {
                                    e.stop_propagation();
                                    on_mouse_down.emit(id);
                                })
                            };
                            node.get_div(on_mouse_down)
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
