use gloo_console::log;
use yew::prelude::*;
use web_sys::{HtmlDivElement, MouseEvent};
use crate::physics::PhysicsWorld;
use crate::types::{Position, Ball};

pub struct UsePhysicsAndDragHandle {
    pub balls: UseStateHandle<Vec<Ball>>,
    pub mouse_position: UseStateHandle<Option<Position>>,
    pub container_width: UseStateHandle<f32>,
    pub container_height: UseStateHandle<f32>,
    pub show_debug_grid: UseStateHandle<bool>,
    pub current_velocity: UseStateHandle<Option<(f32, f32)>>,
    pub on_mouse_down: Callback<MouseEvent>,
    pub on_mouse_move: Callback<MouseEvent>,
    pub on_mouse_up: Callback<MouseEvent>,
    pub on_ball_context_menu: Vec<Callback<MouseEvent>>,
}

#[hook]
pub fn use_physics_and_drag(container_ref: NodeRef) -> UsePhysicsAndDragHandle {
    let physics_world = use_state(|| {
        let world = PhysicsWorld::new();
        std::rc::Rc::new(std::cell::RefCell::new(world))
    });
    let balls = use_state(Vec::new);
    let mouse_position = use_state(|| None);
    let is_dragging = use_state(|| false);
    let container_width = use_state(|| 100.0);
    let container_height = use_state(|| 100.0);
    let show_debug_grid = use_state(|| true);
    let current_velocity = use_state(|| None);
    let pending_grow = use_state(|| None);

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
                // すでに初期化済みなら何もしない
                if !world.is_initialized {
                    world.init_ball_walls(width, height);
                }
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
            handle.forget();
            || ()
        });
    }

    let on_ball_context_menu = {
        let balls = balls.clone();
        let physics_world = physics_world.clone();
        let container_ref = container_ref.clone();
        (0..balls.len()).map(|i| {
            let balls = balls.clone();
            let physics_world = physics_world.clone();
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                // Ball削除
                {
                    let mut world = physics_world.borrow_mut();
                    world.remove_ball(i);
                }
                // Ballリストを更新
                let mut new_balls = (*balls).clone();
                if i < new_balls.len() {
                    new_balls.remove(i);
                    balls.set(new_balls);
                }
            })
        }).collect::<Vec<_>>()
    };

    // Ball成長処理を共通関数に
    fn grow_ball(
        world: &mut PhysicsWorld,
        _balls: &UseStateHandle<Vec<Ball>>,
        pending_grow: &UseStateHandle<Option<(usize, f64)>>,
        mouse_position: &UseStateHandle<Option<Position>>,
    ) {
        if let Some((grow_idx, start_time)) = **pending_grow {
            let now = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now())
                .unwrap_or(0.0);
            let duration = ((now - start_time) / 1000.0).clamp(0.0, 2.0); // 秒
            let min_radius = 10.0f32;
            let max_radius = 60.0f32;
            let duration_f32 = duration as f32;
            let radius = min_radius + (max_radius - min_radius) * (duration_f32 / 2.0);
            world.set_ball_radius(grow_idx, radius);

            // 生成中もマウス座標で速度記録
            if let Some(active_index) = world.active_ball_index {
                if let Some(pos) = (*mouse_position).as_ref() {
                    world.track_drag_position(pos.x, pos.y);
                }
            }
        }
    }

    // Ball成長処理をpending_grow監視で定期実行
    {
        let physics_world = physics_world.clone();
        let balls = balls.clone();
        let pending_grow = pending_grow.clone();
        let mouse_position = mouse_position.clone();
        use_effect_with(pending_grow.clone(), move |pending_grow| {
            if pending_grow.is_some() {
                let physics_world = physics_world.clone();
                let balls = balls.clone();
                let pending_grow = pending_grow.clone();
                let mouse_position = mouse_position.clone();
                let interval = gloo::timers::callback::Interval::new(16, move || {
                    let mut world = physics_world.borrow_mut();
                    grow_ball(&mut world, &balls, &pending_grow, &mouse_position);
                });
                // Box化して返す
                return Box::new(move || drop(interval)) as Box<dyn FnOnce()>;
            }
            // 何もしないBox化クロージャ
            Box::new(|| ()) as Box<dyn FnOnce()>
        });
    }

    let on_mouse_down = {
        let is_dragging = is_dragging.clone();
        let physics_world = physics_world.clone();
        let mouse_position = mouse_position.clone();
        let balls = balls.clone();
        let container_ref = container_ref.clone();
        let container_width = container_width.clone();
        let container_height = container_height.clone();
        let pending_grow = pending_grow.clone();

        Callback::from(move |e: MouseEvent| {
            // 左クリック以外は何もしない
            if e.button() != 0 {
                return;
            }
            if let Some(container) = container_ref.cast::<HtmlDivElement>() {
                let rect = container.get_bounding_client_rect();
                let x = e.client_x() - rect.left() as i32;
                let y = e.client_y() - rect.top() as i32;

                // Ballの上かどうか判定
                let ball_data = (*balls).clone();
                let mut hit_index = None;
                for (i, ball) in ball_data.iter().enumerate() {
                    let dx = x as f32 - ball.position.x as f32;
                    let dy = y as f32 - ball.position.y as f32;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist <= ball.radius {
                        hit_index = Some(i);
                        break;
                    }
                }

                if let Some(active_index) = hit_index {
                    is_dragging.set(true);
                    let mut world = physics_world.borrow_mut();
                    world.velocity_tracker.clear(); // Clear tracker on new drag
                    world.set_dragging(true);
                    log!("on_mouse_down: PhysicsWorld.is_dragging after set_dragging(true): {}", world.get_is_dragging()); // ADD THIS LOG
                    mouse_position.set(Some(Position { x: e.client_x(), y: e.client_y() }));
                    world.set_active_ball(Some(active_index));
                    // 掴んだballの位置を即時更新（既存仕様）
                    world.set_ball_position(active_index, x, y);
                    world.track_drag_position(x, y); // Add initial position to tracker

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
                } else if e.button() == 0 {
                    // 空白クリック: その場でBallを最小サイズで追加し、成長モードに
                    let now = web_sys::window()
                        .and_then(|w| w.performance())
                        .map(|p| p.now())
                        .unwrap_or(0.0);
                    let mut world = physics_world.borrow_mut();
                    world.set_dragging(true);
                    world.velocity_tracker.clear(); // Clear tracker on new ball creation
                    let min_radius = 10.0f32;
                    let idx = world.add_ball(x, y, min_radius);
                    let mut new_balls = (*balls).clone();
                    new_balls.push(Ball {
                        position: Position { x, y },
                        radius: min_radius,
                    });
                    balls.set(new_balls);
                    world.set_active_ball(Some(idx));
                    is_dragging.set(true);
                    log!("on_mouse_down: is_dragging state after set(true): {}", *is_dragging);
                    mouse_position.set(Some(Position { x: e.client_x(), y: e.client_y() }));
                    pending_grow.set(Some((idx, now)));
                    world.track_drag_position(x, y); // Add initial position to tracker
                    log!("on_mouse_down: PhysicsWorld.is_dragging after set_dragging(true) for new ball: {}", world.get_is_dragging()); // ADD THIS LOG
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
        let pending_grow = pending_grow.clone();

        Callback::from(move |e: MouseEvent| {
            if *is_dragging {
                if let Some(container) = container_ref.cast::<HtmlDivElement>() {
                    let rect = container.get_bounding_client_rect();
                    let x = e.client_x() - rect.left() as i32;
                    let y = e.client_y() - rect.top() as i32;

                    mouse_position.set(Some(Position { x, y }));

                    let mut world = physics_world.borrow_mut();
                    // ここで必ずtrack_drag_positionを呼ぶ
                    if let Some(active_index) = world.active_ball_index {
                        world.track_drag_position(x, y);
                    }

                    let mut update_balls = false;
                    if let Some(active_index) = world.active_ball_index {
                        world.set_ball_position(active_index, x, y);
                        // Update current velocity display
                        if let Some(velocity) = world.velocity_tracker.calculate_velocity() {
                            current_velocity.set(Some(velocity));
                        }
                        update_balls = true;
                    }

                    // ballsの更新は最後にまとめて
                    if update_balls {
                        if let Some(active_index) = world.active_ball_index {
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
            }
        })
    };

    let on_mouse_up = {
        let is_dragging = is_dragging.clone();
        let physics_world = physics_world.clone();
        let mouse_position = mouse_position.clone();
        let current_velocity = current_velocity.clone();
        let balls = balls.clone();
        let pending_grow = pending_grow.clone();

        Callback::from(move |_| {
            {
                let mut world = physics_world.borrow_mut();
                // 生成中のボールも含めて、active_ball_indexがSomeなら必ず投げる
                if let Some(active_index) = world.active_ball_index {
                    world.throw_ball(active_index);
                    if let Some(velocity) = world.velocity_tracker.calculate_velocity() {
                        log!("Throwing ball with velocity: ", velocity.0, ", ", velocity.1);
                    } else {
                        log!("Throwing ball with no velocity (velocity_tracker empty).");
                    }
                }
                world.set_dragging(false);
            }
            is_dragging.set(false);
            mouse_position.set(None);
            current_velocity.set(None);
            pending_grow.set(None);
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
        on_ball_context_menu,
    }
}

#[hook]
pub fn use_container_bounds(container_ref: NodeRef) -> UseStateHandle<Option<(i32, i32, i32, i32)>> {
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
