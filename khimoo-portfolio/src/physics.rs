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
        let phys_x = screen_x as f32 / PIXELS_PER_METER;
        let phys_y = screen_y as f32 / PIXELS_PER_METER;
        let phys_radius = screen_radius / PIXELS_PER_METER;
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
        self.ball_handles.len() - 1
    }
    pub fn add_ball_with_container_size(&mut self, container_width: f32, container_height: f32) -> usize {
        let screen_radius = container_width / 10.0;
        let center_x = (container_width / 2.0) as i32;
        let center_y = (container_height / 4.0) as i32;
        self.add_ball(center_x, center_y, screen_radius)
    }
    pub fn init_ball_walls(&mut self, container_width: f32, container_height: f32) {
        self.ball_handles.clear();
        self.ball_radii.clear();
        self.collider_set = ColliderSet::new();
        let phys_width = container_width / PIXELS_PER_METER;
        let phys_height = container_height / PIXELS_PER_METER;
        let wall_thickness = 0.5;
        let top_wall = ColliderBuilder::cuboid(phys_width, wall_thickness / 2.0)
            .translation(vector![0.0, 0.0])
            .friction(0.3)
            .restitution(0.5)
            .build();
        self.collider_set.insert(top_wall);
        let bottom_wall = ColliderBuilder::cuboid(phys_width, wall_thickness)
            .translation(vector![0.0, phys_height])
            .friction(0.3)
            .restitution(0.5)
            .build();
        self.collider_set.insert(bottom_wall);
        let left_wall = ColliderBuilder::cuboid(wall_thickness / 2.0, phys_height)
            .translation(vector![0.0, 0.0])
            .friction(0.3)
            .restitution(0.5)
            .build();
        self.collider_set.insert(left_wall);
        let right_wall = ColliderBuilder::cuboid(wall_thickness / 2.0, phys_height)
            .translation(vector![phys_width, 0.0])
            .friction(0.3)
            .restitution(0.5)
            .build();
        self.collider_set.insert(right_wall);
        let ball_radius = (container_width / 15.0).min(30.0);
        let center_x = (container_width / 2.0) as i32;
        let center_y = (container_height / 2.0) as i32;
        self.add_ball(center_x, center_y, ball_radius);
        self.set_active_ball(Some(0));
        self.is_initialized = true;
    }
    pub fn step(&mut self) -> Vec<Ball> {
        if self.is_initialized {
            let gravity = vector![0.0, 0.0];
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
    pub fn set_ball_position(&mut self, ball_index: usize, screen_x: i32, screen_y: i32) {
        if let Some(&handle) = self.ball_handles.get(ball_index) {
            if let Some(ball) = self.rigid_body_set.get_mut(handle) {
                let phys_x = screen_x as f32 / PIXELS_PER_METER;
                let phys_y = screen_y as f32 / PIXELS_PER_METER;
                ball.set_translation(vector![phys_x, phys_y], true);
            }
        }
    }
    pub fn set_active_ball(&mut self, ball_index: Option<usize>) {
        self.active_ball_index = ball_index;
    }
    pub fn set_dragging(&mut self, is_dragging: bool) {
        self.is_dragging = is_dragging;
        if !self.is_initialized {
            self.is_initialized = true;
        }
        if !is_dragging {
            self.velocity_tracker.clear();
        }
    }
    pub fn track_drag_position(&mut self, screen_x: i32, screen_y: i32) {
        if self.is_dragging {
            self.velocity_tracker.add_position(screen_x, screen_y);
        }
    }
    pub fn throw_ball(&mut self, ball_index: usize) {
        if let Some(velocity) = self.velocity_tracker.calculate_velocity() {
            if let Some(&handle) = self.ball_handles.get(ball_index) {
                if let Some(ball) = self.rigid_body_set.get_mut(handle) {
                    let phys_vx = velocity.0 / PIXELS_PER_METER;
                    let phys_vy = velocity.1 / PIXELS_PER_METER;
                    ball.set_linvel(vector![phys_vx, phys_vy], true);
                }
            }
        }
        self.velocity_tracker.clear();
    }
    pub fn remove_ball(&mut self, index: usize) {
        if index < self.ball_handles.len() {
            let handle = self.ball_handles.remove(index);
            self.ball_radii.remove(index);
            self.rigid_body_set.remove(
                handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true,
            );
        }
    }
    pub fn set_ball_radius(&mut self, index: usize, radius: f32) {
        if let Some(r) = self.ball_radii.get_mut(index) {
            *r = radius;
        }
        if let Some(&handle) = self.ball_handles.get(index) {
            // Remove all colliders attached to this rigid body
            let mut to_remove = vec![];
            for (collider_handle, collider) in self.collider_set.iter() {
                if collider.parent() == Some(handle) {
                    to_remove.push(collider_handle);
                }
            }
            for collider_handle in to_remove {
                self.collider_set.remove(
                    collider_handle,
                    &mut self.island_manager,
                    &mut self.rigid_body_set,
                    true,
                );
            }
            // Add new collider with updated radius
            let phys_radius = radius / PIXELS_PER_METER;
            let new_collider = ColliderBuilder::ball(phys_radius)
                .restitution(0.7)
                .build();
            self.collider_set.insert_with_parent(new_collider, handle, &mut self.rigid_body_set);
        }
    }
}
