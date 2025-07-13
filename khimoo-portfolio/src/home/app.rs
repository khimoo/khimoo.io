use super::components::NodeGraphContainer;
use super::types::{ContainerBound, Position};
use web_sys::HtmlElement;
use yew::prelude::{function_component, html, use_node_ref, use_state, Callback, Html, MouseEvent};
use yew_hooks::{use_effect_update_with_deps, use_measure};

#[function_component(Home)]
pub fn home() -> Html {
    let container_ref = use_node_ref();
    let container_pos_handle = use_state(|| Position { x: 0, y: 0 });
    let container_measure_handle = use_measure(container_ref.clone());
    let mouse_pos_handle = use_state(|| Position::default());

    {
        let node_ref = container_ref.clone();
        let container_pos = container_pos_handle.clone();
        let measure = container_measure_handle.clone();
        use_effect_update_with_deps(
            move |_| {
                if let Some(element) = node_ref.cast::<HtmlElement>() {
                    let rect = element.get_bounding_client_rect();
                    container_pos.set(Position {
                        x: rect.x() as i32,
                        y: rect.y() as i32,
                    });
                }
                || {}
            },
            measure,
        );
    }

    let on_mouse_move = {
        let mouse_pos_handle = mouse_pos_handle.clone();
        Callback::from(move |e: MouseEvent| {
            let pos = Position {
                x: e.client_x(),
                y: e.client_y(),
            };
            mouse_pos_handle.set(pos);
        })
    };

    html! {
        <div onmousemove={on_mouse_move}>
            // Globalな視点でのmouseの座標、要素の座標を渡すぞ！
            <NodeGraphContainer
                container_ref={container_ref}
                container_measure={container_measure_handle.clone()}
                container_bound={
                    ContainerBound {
                        x: container_measure_handle.x as i32 + container_pos_handle.x,
                        y: container_measure_handle.y as i32 + container_pos_handle.y,
                        width: container_measure_handle.width as i32,
                        height: container_measure_handle.height as i32,
                        top: container_measure_handle.top as i32 + container_pos_handle.y,
                        left: container_measure_handle.left as i32 + container_pos_handle.x,
                        bottom: container_measure_handle.bottom as i32 + container_pos_handle.y,
                        right: container_measure_handle.right as i32 + container_pos_handle.x,
                    }
                }
            />
        </div>
    }
}
