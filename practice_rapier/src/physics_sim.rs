use crate::types::{Node, NodeId, NodePosition, Nodes};
use rapier2d::prelude::*;
use std::collections::HashMap;

fn screen_to_physics(pos: &NodePosition) -> Isometry<f32> {
    Isometry::new(vector![pos.x as f32, pos.y as f32], 0.0)
}

fn physics_to_screen(isometry: &Isometry<f32>) -> NodePosition {
    NodePosition {
        x: isometry.translation.x.round() as i32,
        y: isometry.translation.y.round() as i32,
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
}

impl PhysicsWorld {
    pub fn new(initial_nodes: &Nodes) -> Self {
        let mut bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();
        let mut body_map = HashMap::new();

        for node in initial_nodes {
            let rigid_body = RigidBodyBuilder::dynamic()
                .position(screen_to_physics(&node.pos))
                .build();
            let collider = ColliderBuilder::ball(node.radius as f32)
                .restitution(0.7)
                .build();
            let handle = bodies.insert(rigid_body);
            colliders.insert_with_parent(collider, handle, &mut bodies);
            body_map.insert(node.id, handle);
        }

        Self {
            gravity: vector![0.0, 0.0], // y-down gravity
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies,
            colliders,
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            body_map,
        }
    }

    pub fn step(&mut self) {
        let physics_hooks = ();
        let event_handler = ();

        self.integration_parameters.dt = 1.0 / 60.0; // Simulate 60Hz

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
    }

    pub fn get_nodes(&self) -> Nodes {
        self.body_map
            .iter()
            .filter_map(|(id, handle)| {
                let body = &self.bodies[*handle];
                let coll_handles = body.colliders();
                let coll_handle = coll_handles.get(0)?;
                let collider = &self.colliders[*coll_handle];
                let ball = collider.shape().as_ball()?;

                // フィールドアクセスに変更：`ball.radius`
                let radius = ball.radius.round() as i32;

                Some(Node {
                    id: *id,
                    pos: physics_to_screen(body.position()),
                    radius,
                })
            })
            .collect()
    }
    pub fn get_zero(&self) -> NodePosition {
        physics_to_screen(&Isometry::new(vector![0 as f32, 0 as f32], 0.0))
    }

    pub fn set_node_position(&mut self, id: NodeId, pos: &NodePosition) {
        if let Some(handle) = self.body_map.get(&id) {
            if let Some(body) = self.bodies.get_mut(*handle) {
                body.set_position(screen_to_physics(pos), true);
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
