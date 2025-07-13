use super::types::*;
use rapier2d::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub offset: Position,
    pub scale: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            offset: Position::default(),
            scale: 1.0,
        }
    }
}

impl Viewport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn screen_to_physics(&self, screen_pos: &Position) -> Isometry<f32> {
        let world_x = (screen_pos.x - self.offset.x) as f32 / self.scale;
        let world_y = (screen_pos.y - self.offset.y) as f32 / self.scale;
        Isometry::new(vector![world_x, world_y], 0.0)
    }

    pub fn physics_to_screen(&self, physics_pos: &Isometry<f32>) -> Position {
        let screen_x = (physics_pos.translation.x * self.scale) as i32 + self.offset.x;
        let screen_y = (physics_pos.translation.y * self.scale) as i32 + self.offset.y;
        Position {
            x: screen_x,
            y: screen_y,
        }
    }
}

pub struct PhysicsWorld {
    gravity: Vector<f32>,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    body_map: HashMap<NodeId, RigidBodyHandle>,
    anchor_handle: RigidBodyHandle,
    joint_map: HashMap<NodeId, ImpulseJointHandle>,
    node_registry: Rc<RefCell<NodeRegistry>>, // 共有状態
}

impl PhysicsWorld {
    pub fn new(node_registry: Rc<RefCell<NodeRegistry>>, viewport: &Viewport) -> Self {
        let registry = node_registry.borrow();
        let mut bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();
        let mut impulse_joints = ImpulseJointSet::new();
        let mut body_map = HashMap::new();
        let mut joint_map = HashMap::new();

        // アンカー剛体を作成 (画面中央に固定)
        let anchor_rigid_body = RigidBodyBuilder::fixed()
            .position(viewport.screen_to_physics(&Position { x: 400, y: 400 }))
            .build();
        let anchor_handle = bodies.insert(anchor_rigid_body);

        for (id, pos) in &registry.positions {
            let radius = registry.radii.get(id).copied().unwrap_or(30);
            // ノード剛体の作成
            let rigid_body = RigidBodyBuilder::dynamic()
                .position(viewport.screen_to_physics(pos))
                .build();
            let handle = bodies.insert(rigid_body);

            // コライダーの追加
            let collider = ColliderBuilder::ball(radius as f32)
                .restitution(0.7)
                .build();
            colliders.insert_with_parent(collider, handle, &mut bodies);

            body_map.insert(*id, handle);

            // アンカーとノードの間にバネジョイントを作成
            let joint_params = SpringJointBuilder::new(
                0.0,       // 自然長 (rest_length)
                1000000.0, // バネ定数 (stiffness)
                300000.0,  // 減衰係数 (damping)
            )
            .local_anchor1(point![0.0, 0.0]) // アンカー側の接続点
            .local_anchor2(point![0.0, 0.0]) // ノード側の接続点
            .build();

            // ジョイント追加
            let joint_handle = impulse_joints.insert(
                anchor_handle,
                handle,
                joint_params,
                true, // wake_up the bodies
            );

            joint_map.insert(*id, joint_handle);
        }

        Self {
            gravity: vector![0.0, 0.0],
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies,
            colliders,
            impulse_joints,
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            body_map,
            anchor_handle,
            joint_map,
            node_registry: Rc::clone(&node_registry),
        }
    }

    pub fn step(&mut self, viewport: &Viewport) {
        let physics_hooks = ();
        let event_handler = ();

        self.integration_parameters.dt = 1.0 / 120.0;

        let mut pipeline = PhysicsPipeline::new();
        pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            None,
            &physics_hooks,
            &event_handler,
        );

        let mut registry = self.node_registry.borrow_mut();
        for (id, handle) in &self.body_map {
            let body = &self.bodies[*handle];
            if let Some(pos) = registry.positions.get_mut(id) {
                *pos = viewport.physics_to_screen(body.position());
            }
        }
    }

    pub fn set_node_position(&mut self, id: NodeId, pos: &Position, viewport: &Viewport) {
        if let Some(handle) = self.body_map.get(&id) {
            if let Some(body) = self.bodies.get_mut(*handle) {
                body.set_position(viewport.screen_to_physics(pos), true);
            }
        }
    }

    pub fn set_node_kinematic(&mut self, id: NodeId) {
        if let Some(handle) = self.body_map.get(&id) {
            if let Some(body) = self.bodies.get_mut(*handle) {
                body.set_body_type(RigidBodyType::KinematicPositionBased, true);
            }
        }
    }

    pub fn set_node_dynamic(&mut self, id: NodeId) {
        if let Some(handle) = self.body_map.get(&id) {
            if let Some(body) = self.bodies.get_mut(*handle) {
                body.set_body_type(RigidBodyType::Dynamic, true);
            }
        }
    }
}
