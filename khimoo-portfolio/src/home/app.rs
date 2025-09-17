use super::components::NodeGraphContainer;
use super::types::{ContainerBound, Position};
use yew::prelude::{function_component, html, use_node_ref, use_state, Callback, Html, MouseEvent};
use yew_hooks::use_measure;

#[function_component(Home)]
pub fn home() -> Html {
    let container_ref = use_node_ref();
    let container_measure_handle = use_measure(container_ref.clone());
    let mouse_pos_handle = use_state(|| Position::default());

    let on_mouse_move = {
        let mouse_pos_handle = mouse_pos_handle.clone();
        Callback::from(move |e: MouseEvent| {
            let pos = Position {
                x: e.client_x() as f32,
                y: e.client_y() as f32,
            };
            mouse_pos_handle.set(pos);
        })
    };

    // デバッグ情報をコンソールに出力
    web_sys::console::log_1(&format!("Container measure: width={}, height={}, x={}, y={}", 
        container_measure_handle.width, 
        container_measure_handle.height,
        container_measure_handle.x,
        container_measure_handle.y
    ).into());

    html! {
        <div onmousemove={on_mouse_move} style="width: 100vw; height: 100vh;" ref={container_ref.clone()}>
            <NodeGraphContainer
                container_ref={container_ref}
                container_measure={container_measure_handle.clone()}
                container_bound={
                    ContainerBound {
                        x: container_measure_handle.x as f32,
                        y: container_measure_handle.y as f32,
                        width: container_measure_handle.width as f32,
                        height: container_measure_handle.height as f32,
                        top: container_measure_handle.top as f32,
                        left: container_measure_handle.left as f32,
                        bottom: container_measure_handle.bottom as f32,
                        right: container_measure_handle.right as f32,
                    }
                }
            />
        </div>
    }
}
