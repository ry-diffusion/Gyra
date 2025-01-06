use glam::IVec3;
use gyra_net::proto::pallete_delta::{Pallete, PalleteChunk};

pub struct BlockState {
    pub val: i32,
}

impl BlockState {
    pub fn from_i32(val: i32) -> Self {
        Self { val }
    }

    #[inline]
    pub fn get_block_id(&self) -> u16 {
        (self.val & 0xFFFF) as u16
    }

    #[inline]
    pub fn get_state_id(&self) -> u16 {
        ((self.val >> 16) & 0xFFFF) as u16
    }

    pub fn is_air(&self) -> bool {
        self.get_block_id() == 0
    }
}

pub struct Section {
    /// Number of non-air blocks present in the chunk section. "Non-air" is defined as any block other than air, cave air, and void air (in particular, note that fluids such as water are still counted). The client uses this to unload chunks empty chunks. It will keep count of the blocks as they are broken/placed and if this integer reaches 0 the whole chunk section is not rendered, even though there may still be blocks left, if the server sends an incorrect block count.
    pub block_count: i16,
    pub blocks: Vec<BlockState>,
}

/// A chunk is a 16x16x16 block section of the world.
pub struct Chunk {
    /// Each 16-y section of the chunk.
    /// Max 16 sections.
    pub sections: Vec<Section>,
}

impl Chunk {
    fn blocks_per_long(bits_per_block: u8) -> usize {
        64 / bits_per_block as usize
    }

    fn get_block_index(packed: u64, position: usize, bits_per_block: u8) -> usize {
        let bit_position = position * bits_per_block as usize;
        let mask = (1u64 << bits_per_block) - 1;
        ((packed >> bit_position) & mask) as usize
    }

    pub fn from_palette_chunk(chunk: PalleteChunk) -> Self {
        let mut sections = Vec::new();

        for remote_section in chunk.data {
            let mut section = Section {
                block_count: remote_section.block_count,
                blocks: Vec::with_capacity(16 * 16 * 16),
            };

            match remote_section.pallete {
                Pallete::Indirect {
                    bits_per_block,
                    values,
                } => {
                    let blocks_per_long = Self::blocks_per_long(bits_per_block);
                    for packed in remote_section.data_state_id_array {
                        // Extract individual block indices from packed data
                        for i in 0..blocks_per_long {
                            if section.blocks.len() >= 16 * 16 * 16 {
                                break;
                            }

                            let index = Self::get_block_index(packed, i, bits_per_block);
                            if index < values.len() {
                                // Map palette index to actual block state
                                section.blocks.push(BlockState::from_i32(values[index].0));
                            }
                        }
                    }
                }

                _ => {
                    log::warn!("Unsupported pallete type");
                }
            }

            sections.push(section);
        }

        Self { sections }
    }
}


pub fn index_to_coords(index: usize) -> IVec3 {
    let x = index & 0xF;
    let y = (index >> 4) & 0xF;
    let z = (index >> 8) & 0xF;

    IVec3::new(x as i32, y as i32, z as i32)
}