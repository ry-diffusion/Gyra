use std::collections::HashMap;

use chunk::Chunk;
use gyra_net::proto::{pallete_delta::PalleteChunk, play::ChunkData};

use crate::math::ChunkPos;

pub mod chunk;

pub struct World {
    pub chunks: HashMap<ChunkPos, Chunk>,
    pub visible_chunks: Vec<ChunkPos>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            visible_chunks: Vec::new(),
        }
    }

    pub fn import_from_network(&mut self, remote: ChunkData) -> crate::Result<()> {
        let pos = ChunkPos::new(remote.chunk_x, remote.chunk_z);
        let chunk = PalleteChunk::parse(
            &mut remote.data.elements.as_slice(),
            remote.primary_bit_mask.0 as _,
        )?;

        self.chunks.insert(pos, Chunk::from_palette_chunk(chunk));

        Ok(())
    }
}
