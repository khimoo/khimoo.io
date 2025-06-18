use yew::prelude::*;
use web_sys::HtmlDivElement;
use rapier2d::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Properties, PartialEq)]
struct CoordinatesDisplayProps {
    position: Option<Position>,
    container_bounds: Option<(i32, i32, i32, i32)>, // (min_x, max_x, min_y, max_y)
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
        </div>
    }
}

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
    active_ball_index: Option<usize>,
    is_dragging: bool,
    is_initialized: bool,
}

impl PhysicsWorld {
    fn new() -> Self {
        let mut collider_set = ColliderSet::new();

        // Create ground only
        let ground_collider = ColliderBuilder::cuboid(100.0, 0.1)
            .translation(vector![0.0, -10.0])
            .build();
        collider_set.insert(ground_collider);

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
            active_ball_index: None,
            is_dragging: false,
            is_initialized: false,
        }
    }

    fn add_ball(&mut self, x: f32, y: f32) -> usize {
        // Create ball
        let ball_body = RigidBodyBuilder::dynamic()
            .translation(vector![x, y])
            .build();
        let ball_collider = ColliderBuilder::ball(0.5)
            .restitution(0.7)
            .build();
        let ball_handle = self.rigid_body_set.insert(ball_body);
        self.collider_set.insert_with_parent(ball_collider, ball_handle, &mut self.rigid_body_set);

        self.ball_handles.push(ball_handle);
        self.ball_handles.len() - 1 // return index
    }

    fn step(&mut self) -> Vec<(f32, f32)> {
        if !self.is_dragging && self.is_initialized {
            let gravity = vector![0.0, -9.81];
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
            self.ball_handles.iter().map(|&handle| {
                let ball = &self.rigid_body_set[handle];
                let translation = ball.translation();
                (translation.x, translation.y)
            }).collect()
        } else {
            vec![]
        }
    }

    fn set_ball_position(&mut self, ball_index: usize, x: f32, y: f32) {
        if let Some(&handle) = self.ball_handles.get(ball_index) {
            if let Some(ball) = self.rigid_body_set.get_mut(handle) {
                ball.set_translation(vector![x, y], true);
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
    }
}

#[function_component(App)]
fn app() -> Html {
    let mouse_position: UseStateHandle<Option<Position>> = use_state(|| None);
    let ball_positions: UseStateHandle<Vec<Position>> = use_state(|| vec![]);
    let is_dragging: UseStateHandle<bool> = use_state(|| false);
    let container_ref = use_node_ref();
    let container_bounds: UseStateHandle<Option<(i32, i32, i32, i32)>> = use_state(|| None);
    let physics_world = use_state(|| {
        let mut world = PhysicsWorld::new();
        world.add_ball(0.0, 10.0); // 初期位置でballを追加
        world.set_active_ball(Some(0)); // 最初のballをアクティブに
        Rc::new(RefCell::new(world))
    });

    // Update container bounds when ref is available
    {
        let container_ref = container_ref.clone();
        let container_bounds = container_bounds.clone();
        use_effect_with((), move |_| {
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

    let update_positions = {
        let mouse_position = mouse_position.clone();
        let ball_positions = ball_positions.clone();
        let container_ref = container_ref.clone();
        let physics_world = physics_world.clone();
        Callback::from(move |e: MouseEvent| {
            if let Some(container) = container_ref.cast::<HtmlDivElement>() {
                let rect = container.get_bounding_client_rect();
                let x = e.client_x() - rect.left() as i32;
                let y = e.client_y() - rect.top() as i32;

                mouse_position.set(Some(Position {
                    x: e.client_x(),
                    y: e.client_y(),
                }));

                // Update active ball position
                let mut world = physics_world.borrow_mut();
                if let Some(active_index) = world.active_ball_index {
                    let physics_x = (x as f32) / 100.0;
                    let physics_y = (y as f32) / 100.0;
                    world.set_ball_position(active_index, physics_x, physics_y);

                    // Update ball positions state
                    let mut positions = (*ball_positions).clone();
                    if positions.len() <= active_index {
                        positions.resize(active_index + 1, Position { x: 0, y: 0 });
                    }
                    positions[active_index] = Position { x, y };
                    ball_positions.set(positions);
                }
            }
        })
    };

    let on_mouse_down = {
        let is_dragging = is_dragging.clone();
        let update_positions = update_positions.clone();
        let physics_world = physics_world.clone();
        Callback::from(move |e: MouseEvent| {
            is_dragging.set(true);
            physics_world.borrow_mut().set_dragging(true);
            update_positions.emit(e);
        })
    };

    let on_mouse_up = {
        let is_dragging = is_dragging.clone();
        let mouse_position = mouse_position.clone();
        let physics_world = physics_world.clone();
        Callback::from(move |_| {
            is_dragging.set(false);
            physics_world.borrow_mut().set_dragging(false);
            mouse_position.set(None);
        })
    };

    let on_mouse_move = {
        let is_dragging = is_dragging.clone();
        let update_positions = update_positions.clone();
        Callback::from(move |e: MouseEvent| {
            if *is_dragging {
                update_positions.emit(e);
            }
        })
    };

    // Physics update
    {
        let physics_world = physics_world.clone();
        let ball_positions = ball_positions.clone();
        use_effect_with((), move |_| {
            let handle = gloo::timers::callback::Interval::new(16, move || {
                let positions = physics_world.borrow_mut().step();
                let ui_positions: Vec<Position> = positions.iter().map(|(x, y)| Position {
                    x: (x * 100.0) as i32,
                    y: (y * 100.0) as i32,
                }).collect();
                if !ui_positions.is_empty() {
                    ball_positions.set(ui_positions);
                }
            });
            handle.forget();
            || ()
        });
    }

    html! {
        <>
            <h1>{"Mouse Coordinates"}</h1>
            <CoordinatesDisplay
                position={(*mouse_position).clone()}
                container_bounds={(*container_bounds).clone()}
            />
            <div
                ref={container_ref.clone()}
                onmousedown={on_mouse_down}
                onmousemove={on_mouse_move}
                onmouseup={on_mouse_up}
                style="min-height: 100vh; background-color: #f0f0f0; border: 2px solid #ccc; margin: 10px; position: relative;"
            >
                {
                    ball_positions.iter().enumerate().map(|(i, pos)| {
                        html! {
                            <div key={i} style={format!("
                                position: absolute;
                                width: 20px;
                                height: 20px;
                                background-color: black;
                                border-radius: 50%;
                                transform: translate(-50%, -50%);
                                left: {}px;
                                top: {}px;
                            ", pos.x, pos.y)}></div>
                        }
                    }).collect::<Html>()
                }
            </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
