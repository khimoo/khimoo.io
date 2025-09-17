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
            let is_author = registry.is_author_node(*id);
            
            // ノード剛体の作成 - 作者ノードは固定または特別な物理パラメータ
            let rigid_body = if is_author && force_settings.author_fixed_position {
                RigidBodyBuilder::kinematic_position_based()
                    .position(viewport.screen_to_physics(pos))
                    .build()
            } else {
                let damping = if is_author { 
                    force_settings.author_attraction_damping 
                } else { 
                    3.0 
                };
                RigidBodyBuilder::dynamic()
                    .linear_damping(damping)   // 作者ノードは高い減衰
                    .angular_damping(6.0)      // 回転減衰
                    .position(viewport.screen_to_physics(pos))
                    .build()
            };
            let handle = bodies.insert(rigid_body);

            // コライダーの追加 - 作者ノードは大きめ
            let collider = ColliderBuilder::ball(radius as f32)
                .restitution(if is_author { 0.3 } else { 0.7 }) // 作者ノードは低反発
                .build();
            colliders.insert_with_parent(collider, handle, &mut bodies);

            body_map.insert(*id, handle);
        }

        // ノード間のリンクに対するスプリングジョイントを追加
        for (from, to) in &registry.edges {
            if let (Some(&a), Some(&b)) = (body_map.get(from), body_map.get(to)) {
                // 作者ノードとの接続は弱めの力を使用
                let is_author_connection = registry.is_author_node(*from) || registry.is_author_node(*to);
                let spring_strength = if is_author_connection {
                    force_settings.author_attraction_strength * 0.5 // 作者ノードとの接続は弱め
                } else {
                    force_settings.direct_link_strength // 記事間の直接リンクは強め
                };
                
                let damping = if is_author_connection {
                    force_settings.author_attraction_damping * 10.0
                } else {
                    force_settings.direct_link_damping
                };

                let joint_params = SpringJointBuilder::new(
                    0.0,           // 自然長
                    spring_strength, // バネ定数（リンクタイプに応じて調整）
                    damping,       // 減衰（リンクタイプに応じて調整）
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
                    let registry = self.node_registry.borrow();
                    let is_author = registry.is_author_node(id);
                    drop(registry);

                    // 作者ノードが固定の場合はスキップ
                    if is_author && self.force_settings.author_fixed_position {
                        continue;
                    }

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

    // 記事ノードに作者ノードへの引力を適用
    fn apply_author_attraction_forces(&mut self, _viewport: &Viewport) {
        let registry = self.node_registry.borrow();
        let author_node_id = match registry.get_author_node_id() {
            Some(id) => id,
            None => return, // 作者ノードがない場合は何もしない
        };

        let author_pos = match registry.positions.get(&author_node_id) {
            Some(pos) => *pos,
            None => return,
        };
        drop(registry);

        let dt = self.integration_parameters.dt;

        for (id, handle) in self.body_map.clone() {
            // 作者ノード自身はスキップ
            if id == author_node_id {
                continue;
            }

            if let Some(body) = self.bodies.get_mut(handle) {
                if let Some(pos) = self.node_registry.borrow().positions.get(&id) {
                    let dx = (author_pos.x - pos.x) as f32;
                    let dy = (author_pos.y - pos.y) as f32;
                    let distance = (dx * dx + dy * dy).sqrt();

                    // 距離が近すぎる場合はスキップ
                    if distance < 50.0 {
                        continue;
                    }

                    let v = body.linvel();

                    // 作者ノードへの引力（距離に反比例）
                    let force_magnitude = self.force_settings.author_attraction_strength / (distance + 100.0);
                    let fx = (dx / distance) * force_magnitude - self.force_settings.author_attraction_damping * v.x;
                    let fy = (dy / distance) * force_magnitude - self.force_settings.author_attraction_damping * v.y;

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
        // 作者ノードへの引力を適用
        self.apply_author_attraction_forces(viewport);
        // カテゴリベースの引力を適用
        self.apply_category_attraction_forces(viewport);

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

    // 作者ノードを中央に配置
    pub fn center_author_node(&mut self, viewport: &Viewport) {
        let registry = self.node_registry.borrow();
        if let Some(author_id) = registry.get_author_node_id() {
            drop(registry);
            
            let center = Position {
                x: self.container_bound.x + self.container_bound.width / 2.0,
                y: self.container_bound.y + self.container_bound.height / 2.0,
            };
            
            self.set_node_position(author_id, &center, viewport);
            
            // 作者ノードの位置をレジストリにも更新
            if let Some(pos) = self.node_registry.borrow_mut().positions.get_mut(&author_id) {
                *pos = center;
            }
        }
    }

    // 作者ノードの固定/動的切り替え
    pub fn set_author_node_fixed(&mut self, fixed: bool) {
        let registry = self.node_registry.borrow();
        if let Some(author_id) = registry.get_author_node_id() {
            drop(registry);
            
            if fixed {
                self.set_node_kinematic(author_id);
            } else {
                self.set_node_dynamic(author_id);
            }
            
            self.force_settings.author_fixed_position = fixed;
        }
    }

    // デバッグモード用：ジョイント強度を動的に更新
    pub fn update_joint_strengths(&mut self) {
        let registry = self.node_registry.borrow();
        
        // 既存のジョイントを削除
        for handle in &self.edge_joint_handles {
            self.impulse_joints.remove(*handle, true);
        }
        self.edge_joint_handles.clear();

        // 新しい強度でジョイントを再作成
        for (from, to) in &registry.edges {
            if let (Some(&a), Some(&b)) = (self.body_map.get(from), self.body_map.get(to)) {
                let is_author_connection = registry.is_author_node(*from) || registry.is_author_node(*to);
                let spring_strength = if is_author_connection {
                    self.force_settings.author_attraction_strength * 0.5
                } else {
                    self.force_settings.direct_link_strength
                };
                
                let damping = if is_author_connection {
                    self.force_settings.author_attraction_damping * 10.0
                } else {
                    self.force_settings.direct_link_damping
                };

                let joint_params = SpringJointBuilder::new(
                    0.0,
                    spring_strength,
                    damping,
                )
                .local_anchor1(point![0.0, 0.0])
                .local_anchor2(point![0.0, 0.0])
                .build();
                let h = self.impulse_joints.insert(a, b, joint_params, true);
                self.edge_joint_handles.push(h);
            }
        }
        drop(registry);
    }

    // デバッグモード切り替え
    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.force_settings.debug_mode = debug_mode;
        
        // デバッグモード時は接続線を表示
        let mut registry = self.node_registry.borrow_mut();
        registry.set_connection_line_visibility(debug_mode && self.force_settings.show_connection_lines);
    }

    // 接続線の表示/非表示切り替え
    pub fn set_connection_lines_visible(&mut self, visible: bool) {
        self.force_settings.show_connection_lines = visible;
        
        let mut registry = self.node_registry.borrow_mut();
        registry.set_connection_line_visibility(visible && self.force_settings.debug_mode);
    }

    // カテゴリベースの引力を適用
    fn apply_category_attraction_forces(&mut self, _viewport: &Viewport) {
        if !self.force_settings.enable_category_clustering {
            return;
        }

        let registry = self.node_registry.borrow();
        let categories = registry.get_all_categories();
        let dt = self.integration_parameters.dt;

        for category in categories {
            let nodes_in_category = registry.get_nodes_by_category(&category);
            
            // 同じカテゴリのノード間に引力を適用
            for i in 0..nodes_in_category.len() {
                for j in (i + 1)..nodes_in_category.len() {
                    let node1 = nodes_in_category[i];
                    let node2 = nodes_in_category[j];
                    
                    // 作者ノードはスキップ
                    if registry.is_author_node(node1) || registry.is_author_node(node2) {
                        continue;
                    }

                    if let (Some(pos1), Some(pos2)) = (registry.positions.get(&node1), registry.positions.get(&node2)) {
                        let dx = pos2.x - pos1.x;
                        let dy = pos2.y - pos1.y;
                        let distance = (dx * dx + dy * dy).sqrt();

                        // 範囲内の場合のみ引力を適用
                        if distance > 0.0 && distance < self.force_settings.category_attraction_range {
                            let force_magnitude = self.force_settings.category_attraction_strength / (distance + 50.0);
                            
                            if let (Some(&handle1), Some(&handle2)) = (self.body_map.get(&node1), self.body_map.get(&node2)) {
                                let fx = (dx / distance) * force_magnitude * dt;
                                let fy = (dy / distance) * force_magnitude * dt;

                                // Apply force to first body
                                if let Some(body1) = self.bodies.get_mut(handle1) {
                                    body1.apply_impulse(vector![fx, fy], true);
                                }
                                
                                // Apply opposite force to second body
                                if let Some(body2) = self.bodies.get_mut(handle2) {
                                    body2.apply_impulse(vector![-fx, -fy], true);
                                }
                            }
                        }
                    }
                }
            }
        }
        drop(registry);
    }

    // カテゴリクラスタリングの有効/無効切り替え
    pub fn set_category_clustering_enabled(&mut self, enabled: bool) {
        self.force_settings.enable_category_clustering = enabled;
    }

    // ノードサイズを動的に更新（物理コライダーも含む）
    pub fn update_node_size(&mut self, node_id: NodeId, new_radius: i32) {
        // Update registry
        {
            let mut registry = self.node_registry.borrow_mut();
            registry.update_node_radius(node_id, new_radius);
        }

        // Update physics collider
        if let Some(&body_handle) = self.body_map.get(&node_id) {
            // Find and remove the old collider
            let mut collider_handle_to_remove = None;
            for (collider_handle, collider) in self.colliders.iter() {
                if collider.parent() == Some(body_handle) {
                    collider_handle_to_remove = Some(collider_handle);
                    break;
                }
            }

            // Remove old collider and add new one with updated size
            if let Some(old_collider_handle) = collider_handle_to_remove {
                self.colliders.remove(old_collider_handle, &mut self.island_manager, &mut self.bodies, true);
                
                // Create new collider with updated radius
                let is_author = {
                    let registry = self.node_registry.borrow();
                    registry.is_author_node(node_id)
                };
                
                let collider = ColliderBuilder::ball(new_radius as f32)
                    .restitution(if is_author { 0.3 } else { 0.7 })
                    .build();
                self.colliders.insert_with_parent(collider, body_handle, &mut self.bodies);
            }
        }
    }

    // 全ノードのサイズを重要度とリンク数に基づいて更新
    pub fn update_all_node_sizes(&mut self, article_data: &HashMap<NodeId, (Option<u8>, usize)>) {
        let node_ids: Vec<NodeId> = {
            let registry = self.node_registry.borrow();
            registry.positions.keys().cloned().collect()
        };

        for node_id in node_ids {
            if let Some((importance, inbound_count)) = article_data.get(&node_id) {
                let new_radius = {
                    let registry = self.node_registry.borrow();
                    registry.calculate_dynamic_radius(node_id, *importance, *inbound_count)
                };
                self.update_node_size(node_id, new_radius);
            }
        }
    }
}
