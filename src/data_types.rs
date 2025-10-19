use glam::Vec2;

pub struct Transform{
    pub position: Vec2
}

impl Transform {
    pub fn new(x: f32, y: f32) -> Transform {
        Transform{
            position: Vec2::new(x, y)
        }
    }
}