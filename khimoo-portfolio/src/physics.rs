use gloo_console::log;
use rapier2d::prelude::*;
use crate::types::{Ball, Position, VelocityTracker};

pub const PIXELS_PER_METER: f32 = 100.0;
pub const WALL_THICKNESS_PX: f32 = 10.0;

pub struct PhysicsWorld {
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
    pub ball_handles: Vec<RigidBodyHandle>,
    pub ball_radii: Vec<f32>,
    pub active_ball_index: Option<usize>,
    is_dragging: bool,
    pub is_initialized: bool,
    pub velocity_tracker: VelocityTracker,
    pub container_width: f32,
    pub container_height: f32,
}

impl PhysicsWorld {
    pub fn new() -> Self {
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
            container_width: 0.0,
            container_height: 0.0,
        }
    }

    // 画面座標から物理世界座標への変換
    pub fn screen_to_physics(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        (screen_x / PIXELS_PER_METER, screen_y / PIXELS_PER_METER)
    }

    // 物理世界座標から画面座標への変換
    pub fn physics_to_screen(&self, physics_x: f32, physics_y: f32) -> (f32, f32) {
        (physics_x * PIXELS_PER_METER, physics_y * PIXELS_PER_METER)
    }

    pub fn add_ball(&mut self, screen_x: i32, screen_y: i32, screen_radius: f32) -> usize {
        let (physics_x, physics_y) = self.screen_to_physics(screen_x as f32, screen_y as f32);
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![physics_x, physics_y])
            .build();
        let collider = ColliderBuilder::ball(screen_radius / PIXELS_PER_METER)
            .restitution(0.7)
            .build();
        let ball_handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set.insert_with_parent(collider, ball_handle, &mut self.rigid_body_set);
        self.ball_handles.push(ball_handle);
        self.ball_radii.push(screen_radius);
        self.ball_handles.len() - 1
    }

    pub fn init_ball_walls(&mut self, container_width: f32, container_height: f32) {
        self.container_width = container_width;
        self.container_height = container_height;

        let wall_thickness_m = WALL_THICKNESS_PX / PIXELS_PER_METER;

        // 床
        let (ground_x, ground_y) = self.screen_to_physics(container_width / 2.0, container_height + WALL_THICKNESS_PX);
        let ground_collider = ColliderBuilder::cuboid(container_width / 2.0 / PIXELS_PER_METER, wall_thickness_m)
            .translation(vector![ground_x, ground_y])
            .build();
        self.collider_set.insert(ground_collider);

        // 天井
        let (ceiling_x, ceiling_y) = self.screen_to_physics(container_width / 2.0, -WALL_THICKNESS_PX);
        let ceiling_collider = ColliderBuilder::cuboid(container_width / 2.0 / PIXELS_PER_METER, wall_thickness_m)
            .translation(vector![ceiling_x, ceiling_y])
            .build();
        self.collider_set.insert(ceiling_collider);

        // 左壁
        let (left_wall_x, left_wall_y) = self.screen_to_physics(-WALL_THICKNESS_PX, container_height / 2.0);
        let left_wall_collider = ColliderBuilder::cuboid(wall_thickness_m, container_height / 2.0 / PIXELS_PER_METER)
            .translation(vector![left_wall_x, left_wall_y])
            .build();
        self.collider_set.insert(left_wall_collider);

        // 右壁
        let (right_wall_x, right_wall_y) = self.screen_to_physics(container_width + WALL_THICKNESS_PX, container_height / 2.0);
        let right_wall_collider = ColliderBuilder::cuboid(wall_thickness_m, container_height / 2.0 / PIXELS_PER_METER)
            .translation(vector![right_wall_x, right_wall_y])
            .build();
        self.collider_set.insert(right_wall_collider);

        self.is_initialized = true;
    }

    pub fn step(&mut self) -> Vec<Ball> {
        let gravity = vector![0.0, 9.81];
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

        let mut balls_data = Vec::new();
        let container_width = self.container_width;
        let container_height = self.container_height;
        for (i, handle) in self.ball_handles.iter().enumerate() {
            // ball_bodyを可変借用する前に必要な値をコピー
            let current_physics_translation = if let Some(ball_body) = self.rigid_body_set.get(*handle) {
                *ball_body.translation()
            } else {
                continue; // ボールが見つからない場合はスキップ
            };

            let radius = self.ball_radii[i]; // radiusをここで定義
            let (x, y) = self.physics_to_screen(current_physics_translation.x, current_physics_translation.y);

            let mut new_x = x;
            let mut new_y = y;

            if x - radius < 0.0 {
                new_x = radius;
            } else if x + radius > container_width {
                new_x = container_width - radius;
            }

            if y - radius < 0.0 {
                new_y = radius;
            } else if y + radius > container_height {
                new_y = container_height - radius;
            }

            if new_x != x || new_y != y {
                let (physics_new_x, physics_new_y) = self.screen_to_physics(new_x, new_y);
                if let Some(ball_body) = self.rigid_body_set.get_mut(*handle) {
                    ball_body.set_translation(vector![physics_new_x, physics_new_y], true);
                }
            }

            balls_data.push(Ball {
                position: Position {
                    x: x as i32,
                    y: y as i32,
                },
                radius: self.ball_radii[i],
            });
        }
        balls_data
    }

    pub fn set_ball_radius(&mut self, index: usize, new_radius: f32) {
        if let Some(handle) = self.ball_handles.get(index) {
            if let Some(collider) = self.collider_set.get_mut(self.rigid_body_set.get(*handle).unwrap().colliders()[0]) {
                collider.set_shape(SharedShape::ball(new_radius / PIXELS_PER_METER));
                self.ball_radii[index] = new_radius;
            }
        }
    }

    pub fn set_ball_position(&mut self, index: usize, screen_x: i32, screen_y: i32) {
        let (physics_x, physics_y) = self.screen_to_physics(screen_x as f32, screen_y as f32);
        if let Some(handle) = self.ball_handles.get(index) {
            if let Some(ball_body) = self.rigid_body_set.get_mut(*handle) {
                ball_body.set_translation(vector![physics_x, physics_y], true);
                ball_body.set_linvel(vector![0.0, 0.0], true); // ドラッグ中は速度をゼロにする
                ball_body.set_angvel(0.0, true); // ドラッグ中は角速度をゼロにする
            }
        }
    }

    pub fn set_active_ball(&mut self, index: Option<usize>) {
        self.active_ball_index = index;
    }

    pub fn set_dragging(&mut self, dragging: bool) {
        self.is_dragging = dragging;
    }

    pub fn get_is_dragging(&self) -> bool {
        self.is_dragging
    }

    pub fn track_drag_position(&mut self, x: i32, y: i32) {
        self.velocity_tracker.add_position(x, y);
    }

    pub fn throw_ball(&mut self, index: usize) {
        if let Some(handle) = self.ball_handles.get(index) {
            let (vx, vy) = if let Some(v) = self.velocity_tracker.calculate_velocity() { v } else { (0.0, 0.0) };
            let (physics_vx, physics_vy) = self.screen_to_physics(vx, vy);
            if let Some(ball_body) = self.rigid_body_set.get_mut(*handle) {
                log!("Applying velocity: ", physics_vx, ", ", physics_vy);
                ball_body.set_linvel(vector![physics_vx, physics_vy], true);
            }
        }
        self.velocity_tracker.clear();
    }

    pub fn remove_ball(&mut self, index: usize) {
        if let Some(handle) = self.ball_handles.get(index) {
            self.rigid_body_set.remove(
                *handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true,
            );
            self.ball_handles.remove(index);
            self.ball_radii.remove(index);
            // active_ball_indexが削除されたボールを指している場合、Noneにする
            if let Some(active_idx) = self.active_ball_index {
                if active_idx == index {
                    self.active_ball_index = None;
                } else if active_idx > index {
                    self.active_ball_index = Some(active_idx - 1);
                }
            }
        }
    }
}
