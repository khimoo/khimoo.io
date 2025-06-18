use yew::prelude::*;
use web_sys::{HtmlDivElement, MouseEvent};
use rapier2d::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

// Global scale constant for converting between screen pixels and physics meters
const PIXELS_PER_METER: f32 = 100.0;

// --- 構造体定義 (変更なし) ---
#[derive(Clone, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

// Add ball size information
#[derive(Clone, PartialEq)]
struct Ball {
    position: Position,
    radius: f32,
}

// Velocity tracking for throwing
#[derive(Clone)]
struct VelocityTracker {
    positions: Vec<(i32, i32, f64)>, // (x, y, timestamp)
    max_samples: usize,
}

impl VelocityTracker {
    fn new(max_samples: usize) -> Self {
        Self {
            positions: Vec::new(),
            max_samples,
        }
    }

    fn add_position(&mut self, x: i32, y: i32) {
        let timestamp = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        self.positions.push((x, y, timestamp));

        // Keep only the most recent samples
        if self.positions.len() > self.max_samples {
            self.positions.remove(0);
        }
    }

    fn calculate_velocity(&self) -> Option<(f32, f32)> {
        if self.positions.len() < 2 {
            return None;
        }

        let (x1, y1, t1) = self.positions[0];
        let (x2, y2, t2) = self.positions[self.positions.len() - 1];

        let dt = (t2 - t1) / 1000.0; // Convert to seconds
        if dt < 0.01 { // Minimum time threshold
            return None;
        }

        let dx = (x2 - x1) as f32;
        let dy = (y2 - y1) as f32;
        let dt_f32 = dt as f32; // Convert to f32

        let vx = dx / dt_f32;
        let vy = dy / dt_f32;

        Some((vx, vy))
    }

    fn clear(&mut self) {
        self.positions.clear();
    }
}

// Debug grid component
#[derive(Properties, PartialEq)]
struct DebugGridProps {
    container_width: f32,
    container_height: f32,
    grid_spacing: f32, // Grid spacing in meters
}

// --- CoordinatesDisplay コンポーネント (変更なし) ---
#[derive(Properties, PartialEq)]
struct CoordinatesDisplayProps {
    position: Option<Position>,
    container_bounds: Option<(i32, i32, i32, i32)>, // (min_x, max_x, min_y, max_y)
    balls: Vec<Ball>,
    show_debug_grid: bool,
    current_velocity: Option<(f32, f32)>,
    on_toggle_debug_grid: Callback<MouseEvent>,
}

#[function_component(CoordinatesDisplay)]
fn coordinates_display(props: &CoordinatesDisplayProps) -> Html {
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
struct SimulationContainerProps {
    balls: Vec<Ball>,
    on_mouse_down: Callback<MouseEvent>,
    on_mouse_move: Callback<MouseEvent>,
    on_mouse_up: Callback<MouseEvent>,
    container_ref: NodeRef,
    show_debug_grid: bool,
    container_width: f32,
    container_height: f32,
}

#[function_component(SimulationContainer)]
fn simulation_container(props: &SimulationContainerProps) -> Html {
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
                    // Different colors based on ball size
                    let color = if ball.radius > 15.0 { "#ff6b6b" } // Red for large balls
                               else if ball.radius > 10.0 { "#4ecdc4" } // Teal for medium balls
                               else { "#45b7d1" }; // Blue for small balls

                    html! {
                        <div key={i} style={format!("
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
                        ", diameter, diameter, color, ball.position.x, ball.position.y)}></div>
                    }
                }).collect::<Html>()
            }
        </div>
    }
}

// --- PhysicsWorld (変更なし) ---
struct PhysicsWorld {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    ball_handles: Vec<RigidBodyHandle>,
    ball_radii: Vec<f32>, // Store screen radii for each ball
    active_ball_index: Option<usize>,
    is_dragging: bool,
    is_initialized: bool,
    velocity_tracker: VelocityTracker,
}

impl PhysicsWorld {
    fn new() -> Self {
        let collider_set = ColliderSet::new();

        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            ball_handles: Vec::new(),
            ball_radii: Vec::new(),
            active_ball_index: None,
            is_dragging: false,
            is_initialized: false,
            velocity_tracker: VelocityTracker::new(10),
        }
    }

    fn add_ball(&mut self, screen_x: i32, screen_y: i32, screen_radius: f32) -> usize {
        // Convert screen coordinates to physics coordinates
        let phys_x = screen_x as f32 / PIXELS_PER_METER;
        let phys_y = screen_y as f32 / PIXELS_PER_METER;
        let phys_radius = screen_radius / PIXELS_PER_METER;

        // Create ball
        let ball_body = RigidBodyBuilder::dynamic()
            .translation(vector![phys_x, phys_y])
            .build();
        let ball_collider = ColliderBuilder::ball(phys_radius)
            .restitution(0.7)
            .build();
        let ball_handle = self.rigid_body_set.insert(ball_body);
        self.collider_set.insert_with_parent(ball_collider, ball_handle, &mut self.rigid_body_set);

        self.ball_handles.push(ball_handle);
        self.ball_radii.push(screen_radius);
        self.ball_handles.len() - 1 // return index
    }

    fn add_ball_with_container_size(&mut self, container_width: f32, container_height: f32) -> usize {
        // Calculate ball size as 1/10th of container width
        let screen_radius = container_width / 10.0;
        let center_x = (container_width / 2.0) as i32;
        let center_y = (container_height / 4.0) as i32; // Start in upper quarter

        self.add_ball(center_x, center_y, screen_radius)
    }

    fn init_ball_walls(&mut self, container_width: f32, container_height: f32) {
        // Clear existing balls and create new collider set
        self.ball_handles.clear();
        self.ball_radii.clear();
        self.collider_set = ColliderSet::new();

        // Convert container dimensions to physics coordinates
        let phys_width = container_width / PIXELS_PER_METER;
        let phys_height = container_height / PIXELS_PER_METER;
        let wall_thickness = 0.5; // 50cm thick walls

        // Create walls around the container
        // Top wall
        let top_wall = ColliderBuilder::cuboid(phys_width, wall_thickness / 2.0)
            .translation(vector![0.0, 0.0])
            .friction(0.3)
            .restitution(0.5)
            .build();
        self.collider_set.insert(top_wall);

        // Bottom wall
        let bottom_wall = ColliderBuilder::cuboid(phys_width, wall_thickness)
            .translation(vector![0.0, phys_height])
            .friction(0.3)
            .restitution(0.5)
            .build();
        self.collider_set.insert(bottom_wall);

        // Left wall
        let left_wall = ColliderBuilder::cuboid(wall_thickness / 2.0, phys_height)
            .translation(vector![0.0, 0.0])
            .friction(0.3)
            .restitution(0.5)
            .build();
        self.collider_set.insert(left_wall);

        // Right wall
        let right_wall = ColliderBuilder::cuboid(wall_thickness / 2.0, phys_height)
            .translation(vector![phys_width, 0.0])
            .friction(0.3)
            .restitution(0.5)
            .build();
        self.collider_set.insert(right_wall);

        // Add initial ball in the center
        let ball_radius = (container_width / 15.0).min(30.0); // Ball size as 1/15th of width, max 30px
        let center_x = (container_width / 2.0) as i32;
        let center_y = (container_height / 2.0) as i32;

        self.add_ball(center_x, center_y, ball_radius);
        self.set_active_ball(Some(0));

        // Initialize physics engine
        self.is_initialized = true;
    }

    fn step(&mut self) -> Vec<Ball> {
        if self.is_initialized {
            let gravity = vector![0.0, 0.0]; // Downward gravity
            let integration_parameters = IntegrationParameters::default();

            self.physics_pipeline.step(
                &gravity,
                &integration_parameters,
                &mut self.island_manager,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.rigid_body_set,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                &mut self.ccd_solver,
                Some(&mut self.query_pipeline),
                &(),
                &(),
            );
        }

        if self.is_initialized {
            self.ball_handles.iter().enumerate().map(|(i, &handle)| {
                let ball = &self.rigid_body_set[handle];
                let translation = ball.translation();
                let screen_x = (translation.x * PIXELS_PER_METER) as i32;
                let screen_y = (translation.y * PIXELS_PER_METER) as i32;
                let radius = self.ball_radii.get(i).copied().unwrap_or(10.0);

                Ball {
                    position: Position { x: screen_x, y: screen_y },
                    radius,
                }
            }).collect()
        } else {
            vec![]
        }
    }

    fn set_ball_position(&mut self, ball_index: usize, screen_x: i32, screen_y: i32) {
        if let Some(&handle) = self.ball_handles.get(ball_index) {
            if let Some(ball) = self.rigid_body_set.get_mut(handle) {
                let phys_x = screen_x as f32 / PIXELS_PER_METER;
                let phys_y = screen_y as f32 / PIXELS_PER_METER;
                ball.set_translation(vector![phys_x, phys_y], true);
                ball.set_linvel(vector![0.0, 0.0], true);
            }
        }
    }

    fn set_active_ball(&mut self, ball_index: Option<usize>) {
        self.active_ball_index = ball_index;
    }

    fn set_dragging(&mut self, is_dragging: bool) {
        self.is_dragging = is_dragging;
        if !self.is_initialized {
            self.is_initialized = true;
        }
        if !is_dragging {
            // Clear velocity tracker when stopping drag
            self.velocity_tracker.clear();
        }
    }

    fn track_drag_position(&mut self, screen_x: i32, screen_y: i32) {
        if self.is_dragging {
            self.velocity_tracker.add_position(screen_x, screen_y);
        }
    }

    fn throw_ball(&mut self, ball_index: usize) {
        if let Some(velocity) = self.velocity_tracker.calculate_velocity() {
            if let Some(&handle) = self.ball_handles.get(ball_index) {
                if let Some(ball) = self.rigid_body_set.get_mut(handle) {
                    // Convert screen velocity to physics velocity
                    let phys_vx = velocity.0 / PIXELS_PER_METER;
                    let phys_vy = velocity.1 / PIXELS_PER_METER;

                    // Apply velocity to the ball
                    ball.set_linvel(vector![phys_vx, phys_vy], true);
                }
            }
        }
        self.velocity_tracker.clear();
    }
}

// --- ここからリファクタリング箇所 (カスタムフック) ---

/// カスタムフックの戻り値をまとめるための構造体
struct UsePhysicsAndDragHandle {
    balls: UseStateHandle<Vec<Ball>>,
    mouse_position: UseStateHandle<Option<Position>>,
    container_width: UseStateHandle<f32>,
    container_height: UseStateHandle<f32>,
    show_debug_grid: UseStateHandle<bool>,
    current_velocity: UseStateHandle<Option<(f32, f32)>>,
    on_mouse_down: Callback<MouseEvent>,
    on_mouse_move: Callback<MouseEvent>,
    on_mouse_up: Callback<MouseEvent>,
}

/// 物理演算とドラッグ操作に関するロジックをまとめたカスタムフック
#[hook]
fn use_physics_and_drag(container_ref: NodeRef) -> UsePhysicsAndDragHandle {
    let physics_world = use_state(|| {
        let world = PhysicsWorld::new();
        Rc::new(RefCell::new(world))
    });
    let balls = use_state(Vec::new);
    let mouse_position = use_state(|| None);
    let is_dragging = use_state(|| false);
    let container_width = use_state(|| 100.0);
    let container_height = use_state(|| 100.0);
    let show_debug_grid = use_state(|| true);
    let current_velocity = use_state(|| None);

    // Initialize physics world with proper ball sizing when container bounds are available
    {
        let physics_world = physics_world.clone();
        let container_width = container_width.clone();
        let container_height = container_height.clone();
        use_effect_with(container_ref.clone(), move |container_ref| {
            if let Some(container) = container_ref.cast::<HtmlDivElement>() {
                let rect = container.get_bounding_client_rect();
                let width = rect.width() as f32;
                let height = rect.height() as f32;

                // Update container dimensions for debug grid
                container_width.set(width);
                container_height.set(height);

                let mut world = physics_world.borrow_mut();
                // Clear existing balls and add a new one with proper sizing
                world.ball_handles.clear();
                world.ball_radii.clear();
                world.init_ball_walls(width, height);
            }
            || ()
        });
    }

    // 物理エンジンのステップ実行
    {
        let physics_world = physics_world.clone();
        let balls = balls.clone();
        use_effect_with((), move |_| {
            let handle = gloo::timers::callback::Interval::new(16, move || {
                let ball_data = physics_world.borrow_mut().step();
                if !ball_data.is_empty() {
                    balls.set(ball_data);
                }
            });
            // `handle` を `forget` して、コンポーネントが破棄されてもタイマーが止まらないようにする
            handle.forget();
            || ()
        });
    }

    // マウスイベントのコールバック
    let on_mouse_down = {
        let is_dragging = is_dragging.clone();
        let physics_world = physics_world.clone();
        let mouse_position = mouse_position.clone();
        let balls = balls.clone();
        let container_ref = container_ref.clone();

        Callback::from(move |e: MouseEvent| {
            is_dragging.set(true);
            physics_world.borrow_mut().set_dragging(true);

            if let Some(container) = container_ref.cast::<HtmlDivElement>() {
                let rect = container.get_bounding_client_rect();
                let x = e.client_x() - rect.left() as i32;
                let y = e.client_y() - rect.top() as i32;

                mouse_position.set(Some(Position { x: e.client_x(), y: e.client_y() }));

                let mut world = physics_world.borrow_mut();
                if let Some(active_index) = world.active_ball_index {
                    world.set_ball_position(active_index, x, y);
                    world.track_drag_position(x, y);

                    let mut ball_data = (*balls).clone();
                    if ball_data.len() <= active_index {
                        ball_data.resize(active_index + 1, Ball {
                            position: Position { x: 0, y: 0 },
                            radius: 10.0
                        });
                    }
                    ball_data[active_index] = Ball {
                        position: Position { x, y },
                        radius: ball_data[active_index].radius
                    };
                    balls.set(ball_data);
                }
            }
        })
    };

    let on_mouse_move = {
        let is_dragging = is_dragging.clone();
        let physics_world = physics_world.clone();
        let mouse_position = mouse_position.clone();
        let balls = balls.clone();
        let container_ref = container_ref.clone();
        let current_velocity = current_velocity.clone();

        Callback::from(move |e: MouseEvent| {
            if *is_dragging {
                if let Some(container) = container_ref.cast::<HtmlDivElement>() {
                    let rect = container.get_bounding_client_rect();
                    let x = e.client_x() - rect.left() as i32;
                    let y = e.client_y() - rect.top() as i32;

                    mouse_position.set(Some(Position { x: e.client_x(), y: e.client_y() }));

                    let mut world = physics_world.borrow_mut();
                    if let Some(active_index) = world.active_ball_index {
                        world.set_ball_position(active_index, x, y);
                        world.track_drag_position(x, y);

                        // Update current velocity display
                        if let Some(velocity) = world.velocity_tracker.calculate_velocity() {
                            current_velocity.set(Some(velocity));
                        }

                        let mut ball_data = (*balls).clone();
                        if ball_data.len() <= active_index {
                            ball_data.resize(active_index + 1, Ball {
                                position: Position { x: 0, y: 0 },
                                radius: 10.0
                            });
                        }
                        ball_data[active_index] = Ball {
                            position: Position { x, y },
                            radius: ball_data[active_index].radius
                        };
                        balls.set(ball_data);
                    }
                }
            }
        })
    };

    let on_mouse_up = {
        let is_dragging = is_dragging.clone();
        let physics_world = physics_world.clone();
        let mouse_position = mouse_position.clone();
        let current_velocity = current_velocity.clone();

        Callback::from(move |_| {
            let mut world = physics_world.borrow_mut();

            // Throw the ball with current velocity
            if let Some(active_index) = world.active_ball_index {
                world.throw_ball(active_index);
            }

            is_dragging.set(false);
            world.set_dragging(false);
            mouse_position.set(None);
            current_velocity.set(None);
        })
    };

    UsePhysicsAndDragHandle {
        balls,
        mouse_position,
        container_width,
        container_height,
        show_debug_grid,
        current_velocity,
        on_mouse_down,
        on_mouse_move,
        on_mouse_up,
    }
}

/// コンテナの境界を取得するカスタムフック
#[hook]
fn use_container_bounds(container_ref: NodeRef) -> UseStateHandle<Option<(i32, i32, i32, i32)>> {
    let container_bounds = use_state(|| None);

    {
        let container_bounds = container_bounds.clone();
        use_effect_with(container_ref, move |container_ref| {
            if let Some(container) = container_ref.cast::<HtmlDivElement>() {
                let rect = container.get_bounding_client_rect();
                container_bounds.set(Some((
                    rect.left() as i32,
                    rect.right() as i32,
                    rect.top() as i32,
                    rect.bottom() as i32,
                )));
            }
            || ()
        });
    }

    container_bounds
}

/// メインのAppコンポーネント (リファクタリング後)
#[function_component(App)]
fn app() -> Html {
    // 1. DOM要素への参照を作成
    let container_ref = use_node_ref();

    // 2. カスタムフックを呼び出して、ロジックと状態を取得
    let container_bounds = use_container_bounds(container_ref.clone());
    let physics_handle = use_physics_and_drag(container_ref.clone());

    // 3. 取得した状態とコールバックをプレゼンテーションコンポーネントに渡す
    html! {
        <>
            <h1>{"Yew & Rapier2D Physics Simulation"}</h1>

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

            <SimulationContainer
                balls={(*physics_handle.balls).clone()}
                on_mouse_down={physics_handle.on_mouse_down}
                on_mouse_move={physics_handle.on_mouse_move}
                on_mouse_up={physics_handle.on_mouse_up}
                container_ref={container_ref}
                show_debug_grid={(*physics_handle.show_debug_grid).clone()}
                container_width={(*physics_handle.container_width).clone()}
                container_height={(*physics_handle.container_height).clone()}
            />
        </>
    }
}

// --- main (変更なし) ---
fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(DebugGrid)]
fn debug_grid(props: &DebugGridProps) -> Html {
    let grid_spacing_pixels = props.grid_spacing * PIXELS_PER_METER;
    let num_horizontal_lines = (props.container_height / grid_spacing_pixels).ceil() as i32;
    let num_vertical_lines = (props.container_width / grid_spacing_pixels).ceil() as i32;

    html! {
        <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; pointer-events: none; z-index: 1;">
            {
                // Horizontal grid lines
                (0..=num_horizontal_lines).map(|i| {
                    let y = i as f32 * grid_spacing_pixels;
                    let meter_y = y / PIXELS_PER_METER;
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
                    let meter_x = x / PIXELS_PER_METER;
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
