use std::{
    ops::{Deref, Mul},
    simd::Simd,
};

#[repr(C)]
pub struct ChunkPosRepr {
    pub x: i32,
    pub z: i32,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct ChunkPos {
    repr: Simd<i32, 2>,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> Self {
        let repr = Simd::from_array([x, z]);

        Self { repr }
    }

    pub fn form_world_pos(pos: ChunkPos) -> Self {
        Self {
            repr: pos.repr / Simd::splat(16),
        }
    }

    pub fn to_world_pos(&self) -> ChunkPos {
        Self {
            repr: self.repr * Simd::splat(16),
        }
    }
}

impl Deref for ChunkPos {
    type Target = ChunkPosRepr;

    fn deref(&self) -> &Self::Target {
        let array = self.repr.as_array();

        unsafe { &*(array as *const [i32; 2] as *const ChunkPosRepr) }
    }
}
