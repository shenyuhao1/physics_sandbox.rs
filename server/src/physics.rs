use serde::{Deserialize, Serialize};
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
    
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Vec2 {
        let len = self.length();
        if len > 0.0 {
            Vec2::new(self.x / len, self.y / len)
        } else {
            Vec2::zero()
        }
    }

    pub fn dot(&self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn rotate(&self, angle: f32) -> Vec2 {
        let cos = angle.cos();
        let sin = angle.sin();
        Vec2::new(
            self.x * cos - self.y * sin,
            self.x * sin + self.y * cos
        )
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    
    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Shape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBody {
    pub id: u32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub shape: Shape,
    pub mass: f32,
    pub angle: f32,        // 角度（弧度）
    pub angular_velocity: f32, // 角速度
}

impl RigidBody {
    pub fn new_circle(id: u32, position: Vec2, radius: f32, mass: f32) -> Self {
        Self {
            id,
            position,
            velocity: Vec2::zero(),
            shape: Shape::Circle { radius },
            mass,
            angle: 0.0,
            angular_velocity: 0.0,
        }
    }

    pub fn new_rectangle(id: u32, position: Vec2, width: f32, height: f32, mass: f32) -> Self {
        Self {
            id,
            position,
            velocity: Vec2::zero(),
            shape: Shape::Rectangle { width, height },
            mass,
            angle: 0.0,
            angular_velocity: 0.0,
        }
    }

    pub fn radius(&self) -> f32 {
        match self.shape {
            Shape::Circle { radius } => radius,
            Shape::Rectangle { width, height } => (width * width + height * height).sqrt() / 2.0,
        }
    }

    pub fn width(&self) -> f32 {
        match self.shape {
            Shape::Circle { radius } => radius * 2.0,
            Shape::Rectangle { width, .. } => width,
        }
    }

    pub fn height(&self) -> f32 {
        match self.shape {
            Shape::Circle { radius } => radius * 2.0,
            Shape::Rectangle { height, .. } => height,
        }
    }

    // 获取矩形的四个角点（考虑旋转）
    pub fn get_corners(&self) -> Option<[Vec2; 4]> {
        match self.shape {
            Shape::Rectangle { width, height } => {
                let half_width = width / 2.0;
                let half_height = height / 2.0;
                
                // 矩形的四个角（未旋转）
                let corners = [
                    Vec2::new(-half_width, -half_height),
                    Vec2::new(half_width, -half_height),
                    Vec2::new(half_width, half_height),
                    Vec2::new(-half_width, half_height),
                ];
                
                // 应用旋转
                let rotated_corners = [
                    corners[0].rotate(self.angle) + self.position,
                    corners[1].rotate(self.angle) + self.position,
                    corners[2].rotate(self.angle) + self.position,
                    corners[3].rotate(self.angle) + self.position,
                ];
                
                Some(rotated_corners)
            }
            Shape::Circle { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub bodies: Vec<RigidBody>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    ApplyImpulse {
        body_id: u32,
        impulse: Vec2,
    },
    AddRectangle {
        position: Vec2,
        width: f32,
        height: f32,
        mass: f32,
    },
    AddCircle {
        position: Vec2,
        radius: f32,
        mass: f32,
    },
}