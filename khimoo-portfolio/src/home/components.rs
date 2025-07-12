use super::physics_sim::PhysicsWorld;
use super::types::*;
use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;
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

    let node_registry = use_state(|| {
        let mut reg = NodeRegistry::new();
        reg.add_node(NodeId(0), Position { x: 100, y: 150 }, 30, NodeContent::Text("node 0".to_string()));
        reg.add_node(NodeId(1), Position { x: 200, y: 250 }, 50, NodeContent::Text("hello".to_string()));
        Rc::new(RefCell::new(reg))
    });

    let physics_world = use_state(|| {
        Rc::new(RefCell::new(PhysicsWorld::new(Rc::clone(&node_registry))))
    });

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
    let rerender = use_state(|| ());

    {
        let physics_world = physics_world.clone();
        let rerender = rerender.clone();
        use_interval(
            move || {
                let mut world = physics_world.borrow_mut();
                world.step();
                rerender.set(());
            },
            16, // ~60fps
        );
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
                    node_registry.borrow().iter().map(|(id, pos, radius, content)| {
                        let on_mouse_down = {
                            let on_mouse_down = on_mouse_down.clone();
                            let id = *id;
                            Callback::from(move |e: MouseEvent| {
                                e.stop_propagation();
                                on_mouse_down.emit(id);
                            })
                        };
                        html!{
                            <NodeComponent 
                                key={id.0} 
                                id={*id} 
                                pos={*pos} 
                                radius={*radius} 
                                content={content.clone()} 
                                {on_mouse_down}
                            />
                        }
                    }).collect::<Html>()
                }
                <div style={
                    format!("position: absolute;\n                    left: {}px;\n                    top: {}px;\n                    background-color: black;\n                    transform: translate(-50%, -50%);\n                    width: 10px;\n                    height: 10px;\n                    border-radius: 50%;", physics_zero.x, physics_zero.y)}>
                </div>
            </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct NodeProps {
    pub id: NodeId,
    pub pos: Position,
    pub radius: i32,
    pub content: NodeContent,
    pub on_mouse_down: Callback<MouseEvent>,
}

#[function_component(NodeComponent)]
fn node_component(props: &NodeProps) -> Html {
    html! {
        <div
            key={props.id.0.to_string()}
            onmousedown={props.on_mouse_down.clone()}
            style={format!(
                "position: absolute;\n                width: {}px;\n                height: {}px;\n                background-color: black;\n                border-radius: 50%;\n                transform: translate(-50%, -50%);\n                left: {}px;\n                top: {}px;\n                box-shadow: 0 4px 8px rgba(0,0,0,0.2);\n                z-index: 10;\n                display: flex;\n                justify-content: center;\n                align-items: center;",
                2 * props.radius,
                2 * props.radius,
                props.pos.x,
                props.pos.y
            )}
        >
            <div style="max-width: 80%; max-height: 80%; overflow: hidden;">
                {render_content(&props.content)}
            </div>
        </div>
    }
}

fn render_content(content: &NodeContent) -> Html {
    match content {
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