// The Minecraft SMP Format is a format used by Minecraft to store the world data.

use crate::play::ChunkMetadata;
use gyra_codec::coding::{Decoder, Encoder};
use gyra_codec::nibble::NibbleArray;
use gyra_codec::variadic_int::VarInt;
use std::io::Read;

#[derive(Clone, Eq, Copy, Debug, PartialEq, Default)]
pub struct NetworkBlock {
    pub id: u16,      // Block ID (higher 12 bits)
    pub metadata: u8, // Metadata (lower 4 bits)
}

impl NetworkBlock {
    pub const AIR: NetworkBlock = NetworkBlock { id: 0, metadata: 0 };

    fn from_u16(num: u16) -> Self {
        let id = num >> 4; // Extract the higher 12 bits as the block ID
        let metadata = (num & 0xF) as u8; // Extract the lower 4 bits as the metadata
        NetworkBlock { id, metadata }
    }

    #[allow(unused)]
    fn to_u16(&self) -> u16 {
        ((self.id as u16) << 4) | (self.metadata as u16 & 0xF)
    }
}

const WIDTH: usize = 16;
const HEIGHT: usize = 16;
const Z_SIZE: usize = 16;

pub(crate) const ARRAY_SIZE: usize = WIDTH * HEIGHT * Z_SIZE;

/**
 * A chunk section is a 16x16x16 block of the world.
 * It contains the blocks, the skylight and the blocklight.
 */
#[derive(Clone, Debug, PartialEq)]
pub struct ChunkSection {
    // blockID (4) : (4) nibble(metadata)
    pub blocks: Vec<NetworkBlock>,
    pub skylight: NibbleArray,
    pub blocklight: NibbleArray,
    pub count: u32,
}

impl Default for ChunkSection {
    fn default() -> Self {
        Self {
            blocks: vec![NetworkBlock::default(); ARRAY_SIZE],
            skylight: NibbleArray::from_bytes(vec![0; ARRAY_SIZE / 2]),
            blocklight: NibbleArray::from_bytes(vec![0; ARRAY_SIZE / 2]),
            count: 0,
        }
    }
}

impl ChunkSection {
    fn new(blocks: Vec<NetworkBlock>, skylight: NibbleArray, blocklight: NibbleArray) -> Self {
        let count = blocks.iter().filter(|&b| b.id != 0).count() as u32;
        Self {
            blocks,
            skylight,
            blocklight,
            count,
        }
    }

    fn recount(&mut self) {
        self.count = self.blocks.iter().filter(|&b| b.id != 0).count() as u32;
    }

    fn index(x: u16, y: u16, z: u16) -> u16 {
        ((y & 0xf) << 8) | (z << 4) | x
    }

    pub fn metadata(&self, x: u16, y: u16, z: u16) -> u8 {
        self.blocks[ChunkSection::index(x, y, z) as usize].metadata
    }

    pub fn block_id(&self, x: u16, y: u16, z: u16) -> u16 {
        let index = ChunkSection::index(x, y, z) as usize;
        if index >= self.blocks.len() {
            return 0; // AIR
        }

        self.blocks[index].id
    }
}

impl Decoder for ChunkSection {
    // TODO: Implement skylight
    fn decode<R: Read>(reader: &mut R) -> gyra_codec::error::Result<Self> {
        let mut blocks = vec![NetworkBlock::AIR; ARRAY_SIZE];

        for y in 0..=15 {
            for z in 0..=15 {
                for x in 0..=15 {
                    let idx = ChunkSection::index(x, y, z) as usize;
                    let mut buff = [0; 2];
                    reader.read_exact(&mut buff)?;

                    let num = u16::from_le_bytes(buff);
                    let block = NetworkBlock::from_u16(num);

                    blocks[idx] = block;
                }
            }
        }

        let mut blocklight = vec![0; ARRAY_SIZE / 2];
        reader.read_exact(&mut blocklight)?;

        let mut skylight = vec![0; ARRAY_SIZE / 2];
        reader.read_exact(&mut skylight)?;

        Ok(Self::new(
            blocks,
            NibbleArray::from_bytes(blocklight),
            NibbleArray::from_bytes(skylight),
        ))
    }
}

impl Encoder for ChunkSection {
    fn encode<W: std::io::Write>(&self, _writer: &mut W) -> gyra_codec::error::Result<usize> {
        unreachable!("ChunkSection::encode is not a server side operation.")
    }
}

/**
* A chunk is a 16x256x16 block of the world.
* It contains the sections and the biomes.
* Did you know that the world height is limit is caused because of the chunk section counter
* is a nibble? So it only can store 16 values.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkColumn {
    pub sections: [Option<ChunkSection>; 16],
    pub biomes: [u8; 256],
    pub x: i32,
    pub z: i32,
}

impl ChunkColumn {
    fn import_sections(sections: &mut Vec<ChunkSection>, bitmask: u16, column: &mut ChunkColumn) {
        for i in 0..=15 {
            if 0 != (bitmask & (1 << i)) {
                let section = sections.remove(0);
                column.sections[i] = Some(section);
            }
        }
    }

    pub fn from_sections(mut sections: Vec<ChunkSection>, bitmask: u16, x: i32, z: i32) -> Self {
        let mut column = ChunkColumn {
            sections: [const { None }; 16],
            biomes: [0; 256],
            x,
            z,
        };

        Self::import_sections(&mut sections, bitmask, &mut column);

        column
    }

    // Returns the world coordinates of a block in the chunk column
    pub fn block_coordinates(&self, local_x: u32, local_y: u32, local_z: u32) -> (i32, u32, i32) {
        debug_assert!(
            local_x <= 16 && local_y <= 16 && local_z <= 16,
            "the coordinates are out of bounds."
        );

        // Get the chunk's world X and Z coordinates
        let chunk_x = self.x;
        let chunk_z = self.z;

        // Convert local coordinates to world coordinates
        let world_x = chunk_x * 16 + local_x as i32;
        let world_y = local_y; // Y stays the same since it's already in world space
        let world_z = chunk_z * 16 + local_z as i32;

        (world_x, world_y, world_z)
    }

    pub fn get_world_coordinates(&self) -> ((i32, i32, i32), (i32, i32, i32)) {
        let start_x = self.x * 16;
        let start_y = 0; // Always starts at Y=0
        let start_z = self.z * 16;

        let end_x = start_x + 15;
        let end_y = 255; // Always ends at Y=255
        let end_z = start_z + 15;

        ((start_x, start_y, start_z), (end_x, end_y, end_z))
    }

    pub fn block_id_of(&self, x: u16, y: u16, z: u16) -> Option<u16> {
        let section = &self.sections[((y / 16) % 16) as usize];
        Some(section.as_ref()?.block_id(x, y, z))
    }

    pub fn metadata_of(&self, x: u16, y: u16, z: u16) -> u8 {
        let section = &self.sections[(y / 16) as usize];
        section.as_ref().map_or(0, |s| s.metadata(x, y, z))
    }
}

pub fn coord_to_index(x: usize, y: usize, z: usize) -> usize {
    assert!(x <= WIDTH, "x is out of bounds");
    assert!(y <= HEIGHT, "y is out of bounds");
    assert!(z <= Z_SIZE, "z is out of bounds");

    (y * HEIGHT + z) * WIDTH + x
}

#[test]
fn parse_end_stone_block() {
    let num = 1936;
    let block = NetworkBlock::from_u16(num);
    assert_eq!(block.id, 121);
    assert_eq!(block.metadata, 0);
}

#[test]
fn parse_example_chunk_section() {
    // a single section full of end stone
    let mut data = vec![];

    for _ in 0..4096 {
        let raw = 1936u16.to_le_bytes();
        data.push(raw[0]);
        data.push(raw[1]);
    }

    for _ in 0..2048 {
        data.push(0);
    }

    for _ in 0..2048 {
        data.push(0);
    }

    let mut reader = std::io::Cursor::new(data);
    let section = ChunkSection::decode(&mut reader).unwrap();

    let id = section.block_id(1, 1, 1);
    assert_eq!(id, 121);
}
