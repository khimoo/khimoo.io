use super::types::Position;
use rapier2d::prelude::*;

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
        Position { x: screen_x, y: screen_y }
    }
}
