use super::physics_sim::{PhysicsWorld, Viewport};
use super::types::*;
use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;
use yew_hooks::{use_interval, use_window_scroll, UseMeasureState};

#[derive(Properties, PartialEq)]
pub struct NodeGraphContainerProps {
    pub container_ref: NodeRef,
    pub container_measure: UseMeasureState,
    pub container_bound: ContainerBound,
}

#[function_component(NodeGraphContainer)]
pub fn node_graph_container(props: &NodeGraphContainerProps) -> Html {
    let dragged_node_id = use_state(|| None::<NodeId>);
    let viewport = use_state(Viewport::default);

    let node_registry = use_state(|| {
        let mut reg = NodeRegistry::new();
        reg.add_node(
            NodeId(0),
            Position { x: 100, y: 150 },
            30,
            NodeContent::Text("node 0".to_string()),
        );
        reg.add_node(
            NodeId(1),
            Position { x: 200, y: 250 },
            50,
            NodeContent::Text("hello".to_string()),
        );
        reg.add_node(
            NodeId(2),
            Position { x: 350, y: 200 },
            40,
            NodeContent::Text("記事A".to_string()),
        );
        reg.add_node(
            NodeId(3),
            Position { x: 500, y: 300 },
            40,
            NodeContent::Text("記事B".to_string()),
        );
        reg.add_node(
            NodeId(4),
            Position { x: 650, y: 180 },
            40,
            NodeContent::Text("記事C".to_string()),
        );
        // 関連リンク
        reg.add_edge(NodeId(0), NodeId(1));
        reg.add_edge(NodeId(1), NodeId(2));
        reg.add_edge(NodeId(2), NodeId(3));
        reg.add_edge(NodeId(3), NodeId(4));
        reg.add_edge(NodeId(0), NodeId(4));
        Rc::new(RefCell::new(reg))
    });

    let physics_world = use_state(|| {
        Rc::new(RefCell::new(PhysicsWorld::new(
            Rc::clone(&node_registry),
            &viewport,
        )))
    });

    let scroll = use_window_scroll();

    let on_mouse_move = {
        let dragged_node_id = dragged_node_id.clone();
        let physics_world = physics_world.clone();
        let viewport = viewport.clone();
        Callback::from(move |e: MouseEvent| {
            if let Some(id) = *dragged_node_id {
                let mut world = physics_world.borrow_mut();
                let screen_pos = Position {
                    x: e.client_x() + scroll.0 as i32,
                    y: e.client_y() + scroll.1 as i32,
                };
                world.set_node_position(id, &screen_pos, &viewport);
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

    let rerender = use_state(|| ());

    {
        let physics_world = physics_world.clone();
        let viewport = viewport.clone();
        let rerender = rerender.clone();
        use_interval(
            move || {
                let mut world = physics_world.borrow_mut();
                world.step(&viewport);
                rerender.set(());
            },
            16, // ~60fps
        );
    }

    html! {
        <>
            <div
                style="position: static; width: 100vw; height: 100vh; background: #f0f0f0;"
                onmousemove={on_mouse_move}
                onmouseup={on_mouse_up}
                ref={props.container_ref.clone()}
            >
                <h1>{"node_graph"}</h1>
                <p>{ format!("{:?}", *viewport)}</p>
                <p>{ format!("{:?}", props.container_bound)}</p>
                {{
                    // 背景のエッジ描画
                    let reg = node_registry.borrow();
                    html!{
                        <svg style="position: absolute; left: 0; top: 0; width: 100vw; height: 100vh; z-index: 1; pointer-events: none;">
                            {
                                reg.iter_edges().filter_map(|(a, b)| {
                                    let p1 = reg.positions.get(a)?;
                                    let p2 = reg.positions.get(b)?;
                                    Some(html!{
                                        <line
                                            x1={p1.x.to_string()}
                                            y1={p1.y.to_string()}
                                            x2={p2.x.to_string()}
                                            y2={p2.y.to_string()}
                                            stroke="#8a8a8a"
                                            stroke-width="1.5"
                                        />
                                    })
                                }).collect::<Html>()
                            }
                        </svg>
                    }
                }}
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
                2 * props.radius,
                2 * props.radius,
                props.pos.x,
                props.pos.y
            )}
        >
            <div style="max-width: 80%; max-height: 80%; overflow: hidden;">
                {props.content.render_content()}
            </div>
        </div>
    }
}
