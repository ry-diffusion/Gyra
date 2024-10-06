#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
#[repr(C)]
pub struct ChunkVec2 {
    pub x: i32,
    pub z: i32,
}

impl ChunkVec2 {
    pub fn new_local(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn new_global(x: i32, z: i32) -> Self {
        Self {
            x: x >> 4,
            z: z >> 4,
        }
    }

    pub fn as_local(&self) -> (i32, i32) {
        (self.x, self.z)
    }

    pub fn as_global(&self) -> (i32, i32) {
        (self.x << 4, self.z << 4)
    }
}
