use yew::prelude::*;
use crate::hooks::{use_physics_and_drag, use_container_bounds};
use crate::components::{CoordinatesDisplay, SimulationContainer};

#[function_component(App)]
pub fn app() -> Html {
    // 1. DOM要素への参照を作成
    let container_ref = use_node_ref();

    // 2. カスタムフックを呼び出して、ロジックと状態を取得
    let container_bounds = use_container_bounds(container_ref.clone());
    let physics_handle = use_physics_and_drag(container_ref.clone());

    // 3. 取得した状態とコールバックをプレゼンテーションコンポーネントに渡す
    html! {
        <>
            <h1>{"Yew & Rapier2D Physics Simulation"}</h1>
            <SimulationContainer
                balls={(*physics_handle.balls).clone()}
                on_mouse_down={physics_handle.on_mouse_down}
                on_mouse_move={physics_handle.on_mouse_move}
                on_mouse_up={physics_handle.on_mouse_up}
                container_ref={container_ref}
                show_debug_grid={(*physics_handle.show_debug_grid).clone()}
                container_width={*physics_handle.container_width.clone()}
                container_height={*physics_handle.container_height.clone()}
                on_ball_context_menu={physics_handle.on_ball_context_menu.clone()}
            />
            <CoordinatesDisplay
                position={(*physics_handle.mouse_position).clone()}
                container_bounds={(*container_bounds).clone()}
                balls={(*physics_handle.balls).clone()}
                show_debug_grid={(*physics_handle.show_debug_grid).clone()}
                current_velocity={(*physics_handle.current_velocity).clone()}
                on_toggle_debug_grid={{
                    let show_debug_grid = physics_handle.show_debug_grid.clone();
                    Callback::from(move |_| {
                        show_debug_grid.set(!*show_debug_grid);
                    })
                }}
            />
        </>
    }
}
