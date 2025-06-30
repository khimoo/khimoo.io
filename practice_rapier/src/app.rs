use yew::prelude::{Html, html, function_component, use_node_ref, use_state, Callback, MouseEvent};
use yew_hooks::use_measure;
use crate::components::{NodeGraphContainer};
use crate::types::{MousePosition, ContainerMeasure};

#[function_component(App)]
pub fn app() -> Html {
    let mouse_position_handle = use_state(|| MousePosition{x:0,y:0});
    let container_ref = use_node_ref();
    let container_measure_handle = use_measure(container_ref.clone());

    // マウスが動いたときのイベントハンドラ
    let on_mouse_move = {
        let mouse_position_handle = mouse_position_handle.clone();
        Callback::from(move |e: MouseEvent| {
            // ビューポート（ウィンドウ）基準の座標を取得
            mouse_position_handle.set(MousePosition { x: e.client_x(), y: e.client_y() });
        })
    };

    html! {
        <div
            onmousemove={on_mouse_move.clone()}
        >
            <p>{format!("({},{})",mouse_position_handle.x,mouse_position_handle.y)}</p>
            <NodeGraphContainer
                mouse_position={*(mouse_position_handle).clone()}
                container_ref={container_ref}
                container_measure={
                    ContainerMeasure {
                        width: container_measure_handle.width as i32,
                        ..Default::default()
                    }
                }
            />
        </div>
    }
}
