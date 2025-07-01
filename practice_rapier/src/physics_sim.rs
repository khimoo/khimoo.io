use rapier2d::prelude::*;
use crate::types::{ Node, NodePosition, Nodes };

pub struct PhysicsStepResources<'a> {
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
    physics_hooks: Option<&'a dyn PhysicsHooks>,
    event_handler: Option<&'a dyn EventHandler>,
}

impl<'a> PhysicsStepResources<'a> {
    pub fn new(
        physics_hooks: Option<&'a dyn PhysicsHooks>,
        event_handler: Option<&'a dyn EventHandler>,
    ) -> Self {
        Self {
            gravity: vector![0.0, 0.0],
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks,
            event_handler,
        }
    }
    pub fn physics_step() -> Nodes {
        let mut nodes = Nodes::new();
        nodes.insert(0, Node{ id: 0, pos: {NodePosition{x:100, y:150}}});
        nodes.insert(1, Node{ id: 1, pos: {NodePosition{x:300, y:250}}});
        nodes
    }
}
