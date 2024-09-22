use std::io::Read;
use gyra_codec::coding::{Decoder, Encoder};

#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub blocks: Vec<u16>,
    pub block_light: Vec<u8>,
    pub sky_light: Vec<u8>,
}

impl Section {
    pub fn get_block_id(&self, x: u8, y: u8, z: u8) -> u16 {
        self.blocks[(y as usize * 16 * 16) + (z as usize * 16) + x as usize] & 0xFFF
    }
}

impl Decoder for Section {
    fn decode<R: Read>(reader: &mut R) -> gyra_codec::error::Result<Self> {
        let mut blocks = vec![0; 16*16*16];
        
        for _ in 0..16*16*16 {
            blocks.push(u16::decode(reader)?);
        }

        let mut block_light = vec![0; 16*16*8];
        reader.read_exact(&mut block_light)?;

        Ok(Self {
            blocks,
            block_light,
            sky_light: vec![],
        })
    }
}


impl Encoder for Section {
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> gyra_codec::error::Result<usize> {
        todo!()
    }
}