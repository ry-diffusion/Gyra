use std::io::{Read, Write};
use super::error::Result;
use super::variadic_int::VarInt;

pub trait Decoder: Sized {
    fn decode<R: Read>(reader: &mut R) -> Result<Self>;
}

pub trait Encoder {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize>;
}

macro_rules! impl_int {
    ($($t:ty),*) => {
        $(
            impl Decoder for $t {
                fn decode<R: Read>(reader: &mut R) -> Result<Self> {
                    let mut data = [0; std::mem::size_of::<$t>()];
                    reader.read_exact(&mut data)?;
                    Ok(<$t>::from_be_bytes(data))
                }
            }

            impl Encoder for $t {
                fn encode<W: Write>(&self, writer: &mut W) -> Result<usize> {
                    writer.write(&self.to_be_bytes())?;
                    Ok(std::mem::size_of::<$t>())
                }
            }
        )*
    };
}

impl_int!(i8, i16, i32, i64);
impl_int!(u8, u16, u32, u64);

impl Encoder for String {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize> {
        let bytes = self.as_bytes();
        let len = VarInt(bytes.len() as i32).encode(writer)?;
        writer.write(bytes)?;
        Ok(len + bytes.len())
    }
}

impl Decoder for String {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let len = VarInt::decode(reader)?.0 as usize;
        let mut data = vec![0; len];
        reader.read_exact(&mut data)?;
        Ok(String::from_utf8(data)?)
    }
}

impl Encoder for bool {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize> {
        writer.write(&[*self as u8])?;
        Ok(1)
    }
}

impl Decoder for bool {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        let mut data = [0; 1];
        reader.read_exact(&mut data)?;
        Ok(data[0] != 0)
    }
}