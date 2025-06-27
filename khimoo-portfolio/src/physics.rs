use gloo_console::log;
use rapier2d::prelude::*;
use crate::types::{Ball, Position, VelocityTracker};

pub const PIXELS_PER_METER: f32 = 100.0;

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
        }
    }

    pub fn add_ball(&mut self, screen_x: i32, screen_y: i32, screen_radius: f32) -> usize {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![
                screen_x as f32 / PIXELS_PER_METER,
                screen_y as f32 / PIXELS_PER_METER
            ])
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
        // 床
        let ground_collider = ColliderBuilder::cuboid(container_width / 2.0 / PIXELS_PER_METER, 10.0 / PIXELS_PER_METER)
            .translation(vector![container_width / 2.0 / PIXELS_PER_METER, container_height / PIXELS_PER_METER + 10.0 / PIXELS_PER_METER])
            .build();
        self.collider_set.insert(ground_collider);

        // 天井
        let ceiling_collider = ColliderBuilder::cuboid(container_width / 2.0 / PIXELS_PER_METER, 10.0 / PIXELS_PER_METER)
            .translation(vector![container_width / 2.0 / PIXELS_PER_METER, -10.0 / PIXELS_PER_METER])
            .build();
        self.collider_set.insert(ceiling_collider);

        // 左壁
        let left_wall_collider = ColliderBuilder::cuboid(10.0 / PIXELS_PER_METER, container_height / 2.0 / PIXELS_PER_METER)
            .translation(vector![-10.0 / PIXELS_PER_METER, container_height / 2.0 / PIXELS_PER_METER])
            .build();
        self.collider_set.insert(left_wall_collider);

        // 右壁
        let right_wall_collider = ColliderBuilder::cuboid(10.0 / PIXELS_PER_METER, container_height / 2.0 / PIXELS_PER_METER)
            .translation(vector![container_width / PIXELS_PER_METER + 10.0 / PIXELS_PER_METER, container_height / 2.0 / PIXELS_PER_METER])
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
        for (i, handle) in self.ball_handles.iter().enumerate() {
            if let Some(ball_body) = self.rigid_body_set.get(*handle) {
                balls_data.push(Ball {
                    position: Position {
                        x: (ball_body.translation().x * PIXELS_PER_METER) as i32,
                        y: (ball_body.translation().y * PIXELS_PER_METER) as i32,
                    },
                    radius: self.ball_radii[i],
                });
            }
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
        if let Some(handle) = self.ball_handles.get(index) {
            if let Some(ball_body) = self.rigid_body_set.get_mut(*handle) {
                ball_body.set_translation(vector![
                    screen_x as f32 / PIXELS_PER_METER,
                    screen_y as f32 / PIXELS_PER_METER
                ], true);
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
            if let Some(ball_body) = self.rigid_body_set.get_mut(*handle) {
                if let Some((vx, vy)) = self.velocity_tracker.calculate_velocity() {
                    log!("Applying velocity: ", vx, ", ", vy);
                    ball_body.set_linvel(vector![vx / PIXELS_PER_METER, vy / PIXELS_PER_METER], true);
                } else {
                    log!("No velocity to apply.");
                }
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