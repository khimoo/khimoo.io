use yew::prelude::{Html, html, function_component, use_node_ref, use_state, Callback, MouseEvent};
use yew_hooks::use_measure;
use crate::components::{NodeGraphContainer, Interval};
use crate::types::{MousePosition, ContainerMeasure};

#[function_component(App)]
pub fn app() -> Html {
    let container_ref = use_node_ref();
    let container_measure_handle = use_measure(container_ref.clone());


    // use_intervalで時間経過見れる

    html! {
        <div>
            <Interval/>
            <NodeGraphContainer
                container_ref={container_ref}
                container_measure={
                    ContainerMeasure {
                        x: container_measure_handle.x as i32,
                        y: container_measure_handle.y as i32,
                        width: container_measure_handle.width as i32,
                        height: container_measure_handle.height as i32,
                        top: container_measure_handle.top as i32,
                        left: container_measure_handle.left as i32,
                        bottom: container_measure_handle.bottom as i32,
                        right: container_measure_handle.right as i32,
                        ..Default::default()
                    }
                }
            />
        </div>
    }
}
