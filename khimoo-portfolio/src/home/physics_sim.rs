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
        let world_x = (screen_pos.x - self.offset.x) / self.scale;
        let world_y = (screen_pos.y - self.offset.y) / self.scale;
        Isometry::new(vector![world_x, world_y], 0.0)
    }

    pub fn physics_to_screen(&self, physics_pos: &Isometry<f32>) -> Position {
        let screen_x = physics_pos.translation.x * self.scale + self.offset.x;
        let screen_y = physics_pos.translation.y * self.scale + self.offset.y;
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
    node_registry: Rc<RefCell<NodeRegistry>>, // 共有状態
    edge_joint_handles: Vec<ImpulseJointHandle>,
    force_settings: ForceSettings,
    container_bound: ContainerBound, // 追加: コンテナ境界を保持
}

impl PhysicsWorld {
    pub fn new(node_registry: Rc<RefCell<NodeRegistry>>, viewport: &Viewport, force_settings: ForceSettings, container_bound: ContainerBound) -> Self {
        let registry = node_registry.borrow();
        let mut bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();
        let mut impulse_joints = ImpulseJointSet::new();
        let mut body_map = HashMap::new();
        let mut edge_joint_handles = Vec::new();

        for (id, pos) in &registry.positions {
            let radius = registry.radii.get(id).copied().unwrap_or(30);
            // ノード剛体の作成
            let rigid_body = RigidBodyBuilder::dynamic()
                .linear_damping(3.0)   // 全身等方粘性（並進）
                .angular_damping(6.0)  // 全身等方粘性（回転）
                .position(viewport.screen_to_physics(pos))
                .build();
            let handle = bodies.insert(rigid_body);

            // コライダーの追加
            let collider = ColliderBuilder::ball(radius as f32)
                .restitution(0.7)
                .build();
            colliders.insert_with_parent(collider, handle, &mut bodies);

            body_map.insert(*id, handle);
        }

        // ノード間のリンクに対するスプリングジョイントを追加
        for (from, to) in &registry.edges {
            if let (Some(&a), Some(&b)) = (body_map.get(from), body_map.get(to)) {
                let joint_params = SpringJointBuilder::new(
                    0.0,     // 自然長
                    force_settings.link_strength,  // バネ定数
                    200.0,   // 減衰
                )
                .local_anchor1(point![0.0, 0.0])
                .local_anchor2(point![0.0, 0.0])
                .build();
                let h = impulse_joints.insert(a, b, joint_params, true);
                edge_joint_handles.push(h);
            }
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
            node_registry: Rc::clone(&node_registry),
            edge_joint_handles,
            force_settings,
            container_bound, // 追加
        }
    }

    // 各ノードに中心へ向かう力を適用（動的計算: ContainerBoundの中心）
    fn apply_center_forces(&mut self, _viewport: &Viewport) {
        // ContainerBound の中心座標を動的に計算
        let center = Position {
            x: self.container_bound.x + self.container_bound.width / 2.0,
            y: self.container_bound.y + self.container_bound.height / 2.0,
        };
        let dt = self.integration_parameters.dt;

        for (id, handle) in self.body_map.clone() {
            if let Some(body) = self.bodies.get_mut(handle) {
                if let Some(pos) = self.node_registry.borrow().positions.get(&id) {
                    let dx = (center.x - pos.x) as f32;
                    let dy = (center.y - pos.y) as f32;
                    let v = body.linvel();

                    let fx = self.force_settings.center_strength * dx
                        - self.force_settings.center_damping * v.x;
                    let fy = self.force_settings.center_strength * dy
                        - self.force_settings.center_damping * v.y;

                    let impulse = vector![fx * dt, fy * dt];
                    body.apply_impulse(impulse, true);
                }
            }
        }
    }

    // ノード間の反発力を計算して適用
    fn apply_repulsion_forces(&mut self, _viewport: &Viewport) {
        let registry = self.node_registry.borrow();
        let mut forces = HashMap::new();

        // 全てのノードペアに対して反発力を計算
        for (id1, pos1) in &registry.positions {
            for (id2, pos2) in &registry.positions {
                if id1 == id2 {
                    continue;
                }

                let dx = pos2.x - pos1.x;
                let dy = pos2.y - pos1.y;
                let distance = ((dx * dx + dy * dy) as f32).sqrt();

                if distance < 1.0 {
                    continue; // 距離が近すぎる場合はスキップ
                }

                let radius1 = registry.radii.get(id1).copied().unwrap_or(30) as f32;
                let radius2 = registry.radii.get(id2).copied().unwrap_or(30) as f32;
                let min_distance = radius1 + radius2 + self.force_settings.repulsion_min_distance; // 最小距離（半径 + 余白）

                if distance < min_distance {
                    // 反発力の強さ（距離が近いほど強い）
                    let force_magnitude = self.force_settings.repulsion_strength * (min_distance - distance) / min_distance;

                    // 力の方向（id1からid2への方向）
                    let force_x = (dx as f32 / distance) * force_magnitude;
                    let force_y = (dy as f32 / distance) * force_magnitude;

                    // id1に-id2方向の力を、id2にid1方向の力を適用
                    *forces.entry(*id1).or_insert((0.0, 0.0)) =
                        (forces.get(id1).unwrap_or(&(0.0, 0.0)).0 - force_x,
                         forces.get(id1).unwrap_or(&(0.0, 0.0)).1 - force_y);

                    *forces.entry(*id2).or_insert((0.0, 0.0)) =
                        (forces.get(id2).unwrap_or(&(0.0, 0.0)).0 + force_x,
                         forces.get(id2).unwrap_or(&(0.0, 0.0)).1 + force_y);
                }
            }
        }

        // 計算した力を各ノードに適用
        for (id, (fx, fy)) in forces {
            if let Some(&handle) = self.body_map.get(&id) {
                if let Some(body) = self.bodies.get_mut(handle) {
                    let impulse = vector![fx, fy];
                    body.apply_impulse(impulse, true);
                }
            }
        }
    }

    // 力の設定を更新
    pub fn update_force_settings(&mut self, new_settings: ForceSettings) {
        self.force_settings = new_settings;
    }

    // コンテナ境界を更新
    pub fn update_container_bound(&mut self, new_bound: ContainerBound) {
        self.container_bound = new_bound;
    }

    pub fn step(&mut self, viewport: &Viewport) {
        let physics_hooks = ();
        let event_handler = ();

        self.integration_parameters.dt = 1.0 / 12.0;

        // 中心力を適用
        self.apply_center_forces(viewport);
        // 反発力を適用
        self.apply_repulsion_forces(viewport);

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
