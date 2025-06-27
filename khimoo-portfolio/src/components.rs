use yew::prelude::*;
use web_sys::MouseEvent;
use crate::types::{Position, Ball};

#[derive(Properties, PartialEq)]
pub struct DebugGridProps {
    pub container_width: f32,
    pub container_height: f32,
    pub grid_spacing: f32, // Grid spacing in meters
}

#[derive(Properties, PartialEq)]
pub struct CoordinatesDisplayProps {
    pub position: Option<Position>,
    pub container_bounds: Option<(i32, i32, i32, i32)>,
    pub balls: Vec<Ball>,
    pub show_debug_grid: bool,
    pub current_velocity: Option<(f32, f32)>,
    pub on_toggle_debug_grid: Callback<MouseEvent>,
}

#[function_component(CoordinatesDisplay)]
pub fn coordinates_display(props: &CoordinatesDisplayProps) -> Html {
    html! {
        <div>
            <h3>{"Mouse Coordinates"}</h3>
            <p>{
                match &props.position {
                    Some(pos) => format!("X: {}, Y: {}", pos.x, pos.y),
                    None => "Click and drag to see coordinates".to_string(),
                }
            }</p>
            <h3>{"Drag Velocity"}</h3>
            <p>{
                match &props.current_velocity {
                    Some((vx, vy)) => {
                        let speed = (vx * vx + vy * vy).sqrt();
                        format!("Velocity: ({:.1}, {:.1}) px/s, Speed: {:.1} px/s", vx, vy, speed)
                    },
                    None => "Drag to see velocity".to_string(),
                }
            }</p>
            <h3>{"Container Bounds"}</h3>
            <p>{
                match &props.container_bounds {
                    Some((min_x, max_x, min_y, max_y)) => format!(
                        "X: {} to {}, Y: {} to {}",
                        min_x, max_x, min_y, max_y
                    ),
                    None => "Loading...".to_string(),
                }
            }</p>
            <h3>{"Ball Information"}</h3>
            <p>{format!("Number of balls: {}", props.balls.len())}</p>
            {
                props.balls.iter().enumerate().map(|(i, ball)| {
                    html! {
                        <p key={i}>{
                            format!("Ball {}: Position({}, {}), Radius: {:.1}px",
                                i, ball.position.x, ball.position.y, ball.radius)
                        }</p>
                    }
                }).collect::<Html>()
            }
            <h3>{"Physics Setup"}</h3>
            <p>{"Walls: 4 walls around container with friction and restitution"}</p>
            <p>{"Gravity: 9.81 m/s² downward"}</p>
            <p>{"Physics Engine: "}{if props.balls.is_empty() { "Initializing..." } else { "Running" }}</p>
            <h3>{"Debug Controls"}</h3>
            <button
                onclick={props.on_toggle_debug_grid.clone()}
                style="padding: 8px 16px; background-color: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer;"
            >
                {if props.show_debug_grid { "Hide Debug Grid" } else { "Show Debug Grid" }}
            </button>
            <p><small>{"Scale: 100 pixels = 1 meter"}</small></p>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SimulationContainerProps {
    pub balls: Vec<Ball>,
    pub on_mouse_down: Callback<MouseEvent>,
    pub on_mouse_move: Callback<MouseEvent>,
    pub on_mouse_up: Callback<MouseEvent>,
    pub container_ref: NodeRef,
    pub show_debug_grid: bool,
    pub container_width: f32,
    pub container_height: f32,
    pub on_ball_context_menu: Vec<Callback<MouseEvent>>,
}

#[function_component(SimulationContainer)]
pub fn simulation_container(props: &SimulationContainerProps) -> Html {
    html! {
        <div
            ref={props.container_ref.clone()}
            onmousedown={props.on_mouse_down.clone()}
            onmousemove={props.on_mouse_move.clone()}
            onmouseup={props.on_mouse_up.clone()}
            style="min-height: 100vh; background-color: #f0f0f0; border: 4px solid #333; margin: 10px; position: relative; box-shadow: inset 0 0 20px rgba(0,0,0,0.1);"
        >
            if props.show_debug_grid {
                <DebugGrid
                    container_width={props.container_width}
                    container_height={props.container_height}
                    grid_spacing={1.0}
                />
            }
            {
                props.balls.iter().enumerate().map(|(i, ball)| {
                    let diameter = ball.radius * 2.0;
                    let color = if ball.radius > 15.0 { "#ff6b6b" }
                               else if ball.radius > 10.0 { "#4ecdc4" }
                               else { "#45b7d1" };
                    html! {
                        <div key={i}
                            style={format!("
                                position: absolute;
                                width: {}px;
                                height: {}px;
                                background-color: {};
                                border-radius: 50%;
                                transform: translate(-50%, -50%);
                                left: {}px;
                                top: {}px;
                                box-shadow: 0 4px 8px rgba(0,0,0,0.2);
                                z-index: 10;
                            ", diameter, diameter, color, ball.position.x, ball.position.y)}
                            oncontextmenu={props.on_ball_context_menu.get(i).cloned().unwrap_or_default()}
                        ></div>
                    }
                }).collect::<Html>()
            }
        </div>
    }
}

#[function_component(DebugGrid)]
pub fn debug_grid(props: &DebugGridProps) -> Html {
    let grid_spacing_pixels = props.grid_spacing * 100.0;
    let num_horizontal_lines = (props.container_height / grid_spacing_pixels).ceil() as i32;
    let num_vertical_lines = (props.container_width / grid_spacing_pixels).ceil() as i32;

    html! {
        <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: 1;">
            {
                // Horizontal grid lines
                (0..=num_horizontal_lines).map(|i| {
                    let y = i as f32 * grid_spacing_pixels;
                    let meter_y = y / 100.0;
                    html! {
                        <div key={format!("h-{}", i)} style={format!("
                            position: absolute;
                            left: 0;
                            top: {}px;
                            width: 100%;
                            height: 1px;
                            background-color: rgba(0, 255, 0, 0.3);
                            z-index: 1;
                        ", y)}>
                            <span style={format!("
                                position: absolute;
                                left: 5px;
                                top: -15px;
                                font-size: 10px;
                                color: green;
                                background-color: rgba(255, 255, 255, 0.8);
                                padding: 1px 3px;
                                border-radius: 2px;
                            ")}>
                                {format!("{:.1}m", meter_y)}
                            </span>
                        </div>
                    }
                }).collect::<Html>()
            }
            {
                // Vertical grid lines
                (0..=num_vertical_lines).map(|i| {
                    let x = i as f32 * grid_spacing_pixels;
                    let meter_x = x / 100.0;
                    html! {
                        <div key={format!("v-{}", i)} style={format!("
                            position: absolute;
                            top: 0;
                            left: {}px;
                            width: 1px;
                            height: 100%;
                            background-color: rgba(0, 255, 0, 0.3);
                            z-index: 1;
                        ", x)}>
                            <span style={format!("
                                position: absolute;
                                top: 5px;
                                left: -20px;
                                font-size: 10px;
                                color: green;
                                background-color: rgba(255, 255, 255, 0.8);
                                padding: 1px 3px;
                                border-radius: 2px;
                                transform: rotate(-90deg);
                                transform-origin: center;
                            ")}>
                                {format!("{:.1}m", meter_x)}
                            </span>
                        </div>
                    }
                }).collect::<Html>()
            }
            // Origin marker
            <div style="
                position: absolute;
                left: 0;
                top: 0;
                width: 10px;
                height: 10px;
                background-color: red;
                border-radius: 50%;
                z-index: 2;
                border: 2px solid white;
            ">
                <span style="
                    position: absolute;
                    top: -20px;
                    left: -10px;
                    font-size: 10px;
                    color: red;
                    background-color: rgba(255, 255, 255, 0.9);
                    padding: 1px 3px;
                    border-radius: 2px;
                    white-space: nowrap;
                ">
                    {"Origin (0,0)"}
                </span>
            </div>
        </div>
    }
} 