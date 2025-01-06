use crate::coding::{Decoder, Encoder};
use crate::error::Result;
use std::io::Read;

// Encoded as an unsigned 128-bit integer (or two unsigned 64-bit integers: the most significant 64 bits and then the least significant 64 bits)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Uuid {
    pub most_significant_bits: u64,
    pub least_significant_bits: u64,
}

impl Decoder for Uuid {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let mut most_significant_bits = [0; 8];
        let mut least_significant_bits = [0; 8];

        reader.read_exact(&mut most_significant_bits)?;
        reader.read_exact(&mut least_significant_bits)?;

        Ok(Uuid {
            most_significant_bits: u64::from_be_bytes(most_significant_bits),
            least_significant_bits: u64::from_be_bytes(least_significant_bits),
        })
    }
}

impl Encoder for Uuid {
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<usize> {
        writer.write_all(&self.most_significant_bits.to_be_bytes())?;
        writer.write_all(&self.least_significant_bits.to_be_bytes())?;

        Ok(16)
    }
}
