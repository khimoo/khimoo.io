// 型定義と物理演算の補助構造体

#[derive(Clone, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, PartialEq)]
pub struct Ball {
    pub position: Position,
    pub radius: f32,
}

#[derive(Clone)]
pub struct VelocityTracker {
    positions: Vec<(i32, i32, f64)>, // (x, y, timestamp)
    max_samples: usize,
}

impl VelocityTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            positions: Vec::new(),
            max_samples,
        }
    }
    pub fn add_position(&mut self, x: i32, y: i32) {
        let timestamp = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        self.positions.push((x, y, timestamp));
        if self.positions.len() > self.max_samples {
            self.positions.remove(0);
        }
    }
    pub fn calculate_velocity(&self) -> Option<(f32, f32)> {
        if self.positions.len() < 2 {
            return None;
        }
        let (x1, y1, t1) = self.positions[0];
        let (x2, y2, t2) = self.positions[self.positions.len() - 1];
        let dt = (t2 - t1) / 1000.0;
        if dt < 0.01 {
            return None;
        }
        let dx = (x2 - x1) as f32;
        let dy = (y2 - y1) as f32;
        let dt_f32 = dt as f32;
        let vx = dx / dt_f32;
        let vy = dy / dt_f32;
        Some((vx, vy))
    }
    pub fn clear(&mut self) {
        self.positions.clear();
    }
} 