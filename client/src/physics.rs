use serde::{Deserialize, Serialize};
#![allow(dead_code)]
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
}

impl RigidBody {
    pub fn new_circle(id: u32, position: Vec2, radius: f32, mass: f32) -> Self {
        Self {
            id,
            position,
            velocity: Vec2::zero(),
            shape: Shape::Circle { radius },
            mass,
        }
    }

    pub fn new_rectangle(id: u32, position: Vec2, width: f32, height: f32, mass: f32) -> Self {
        Self {
            id,
            position,
            velocity: Vec2::zero(),
            shape: Shape::Rectangle { width, height },
            mass,
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
}