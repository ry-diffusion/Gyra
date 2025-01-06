use glam::Vec3A;

#[derive(Debug, PartialEq)]
pub struct Player {
    pub position: Vec3A,
    pub yaw: f32,
    pub pitch: f32,
}
