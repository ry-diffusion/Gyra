// The Minecraft SMP Format is a format used by Minecraft to store the world data.

use gyra_codec::coding::{Decoder, Encoder};
use gyra_codec::nibble::NibbleArray;
use std::io::Read;

const WIDTH: usize = 16;
const HEIGHT: usize = 16;
const Z_SIZE: usize = 16;

const ARRAY_SIZE: usize = WIDTH * HEIGHT * Z_SIZE;

/**
 * A chunk section is a 16x16x16 block of the world.
 * It contains the blocks, the skylight and the blocklight.
 */
#[derive(Clone, Debug, PartialEq)]
pub struct ChunkSection {
    // blockID (4) : (4) nibble(metadata)
    pub blocks: Vec<u16>,
    pub skylight: NibbleArray,
    pub blocklight: NibbleArray,
    pub count: u32,
}

impl ChunkSection {
    fn default() -> Self {
        Self {
            blocks: vec![0; ARRAY_SIZE],
            skylight: NibbleArray::from_bytes(vec![0; ARRAY_SIZE / 2]),
            blocklight: NibbleArray::from_bytes(vec![0; ARRAY_SIZE / 2]),
            count: 0,
        }
    }

    fn new(blocks: Vec<u16>, skylight: NibbleArray, blocklight: NibbleArray) -> Self {
        let count = blocks.iter().filter(|&b| *b != 0).count() as u32;
        Self {
            blocks,
            skylight,
            blocklight,
            count,
        }
    }

    fn recount(&mut self) {
        self.count = self.blocks.iter().filter(|&b| *b != 0).count() as u32;
    }

    fn index(x: u32, y: u32, z: u32) -> u32 {
        ((y & 0xf) << 8) | (z << 4) | x
    }

    pub fn metadata(&self, x: u32, y: u32, z: u32) -> u16 {
        self.blocks[ChunkSection::index(x, y, z) as usize] & 0xF
    }

    pub fn block_id(&self, x: u32, y: u32, z: u32) -> u16 {
        let index = ChunkSection::index(x, y, z) as usize;
        self.blocks[index] 
    }

    fn from_reader(reader: &mut impl Read) -> Self {
        let mut cs = ChunkSection::default();

        for i in 0..ARRAY_SIZE {
            cs.blocks[i] = u16::decode(reader).unwrap();
        }

        cs.recount();
        cs
    }
}

impl Decoder for ChunkSection {
    fn decode<R: Read>(reader: &mut R) -> gyra_codec::error::Result<Self> {
        let mut blocks = vec![0; ARRAY_SIZE];

        for i in 0..ARRAY_SIZE {
            blocks[i] = u16::decode(reader)?;
        }

        let mut blocklight = vec![0; ARRAY_SIZE / 2];
        reader.read_exact(&mut blocklight)?;

        let mut skylight = vec![0; ARRAY_SIZE / 2];
        reader.read_exact(&mut skylight)?;

        Ok(Self::new(
            blocks,
            NibbleArray::from_bytes(skylight),
            NibbleArray::from_bytes(blocklight),
        ))
    }
}

impl Encoder for ChunkSection {
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> gyra_codec::error::Result<usize> {
        todo!();
        Ok(0)
    }
}

/**
* A chunk is a 16x16x256 block of the world.
* It contains the sections and the biomes.
* Did you know that the world height is limit is caused because of the chunk section counter
* is a nibble? So it only can store 16 values.
*/
pub struct Chunk {
    pub sections: Vec<ChunkSection>,
    pub biomes: Vec<u8>,
}

impl Chunk {
    fn new() -> Self {
        Self {
            sections: vec![ChunkSection::default(); 16],
            biomes: vec![0; 256],
        }
    }

    fn get_block_id(&self, x: u32, y: u32, z: u32) -> u8 {
        let section = &self.sections[(y / 16) as usize];
        section.block_id(x, y, z) as u8
    }

    fn get_metadata(&self, x: u32, y: u32, z: u32) -> u8 {
        let section = &self.sections[(y / 16) as usize];
        section.metadata(x, y, z) as u8
    }
}

pub fn coord_to_index(x: usize, y: usize, z: usize) -> usize {
    assert!(x <= WIDTH, "x is out of bounds");
    assert!(y <= HEIGHT, "y is out of bounds");
    assert!(z <= Z_SIZE, "z is out of bounds");

    (y * HEIGHT + z) * WIDTH + x
}
