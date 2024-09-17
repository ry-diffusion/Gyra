use crate::coding::{Decoder, Encoder};
use crate::error::{CodecError, Result, VarIntError};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VarInt(pub i32);

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        VarInt(value)
    }
}

impl From<u32> for VarInt {
    fn from(value: u32) -> Self {
        VarInt { 0: value as i32 }
    }
}

impl From<VarInt> for i32 {
    fn from(value: VarInt) -> Self {
        value.0
    }
}

impl From<VarInt> for u32 {
    fn from(value: VarInt) -> Self {
        value.0 as u32
    }
}

impl Decoder for VarInt {
    fn decode<R: Read>(reader: &mut R) -> Result<Self> {
        // let mut val = 0;
        // for i in 0..5 {
        //     let mut byte = [0];
        //
        //     reader.read(&mut byte)?;
        //
        //     val |= (i32::from(byte[0]) & 0b01111111) << (i * 7);
        //     if byte[0] & 0b10000000 == 0 {
        //         return Ok(VarInt(val));
        //     }
        // }

        const PART: u32 = 0x7F;
        let mut size = 0;
        let mut val = 0u32;
        loop {
            let mut byte = [0];

            reader.read_exact(&mut byte)?;

            let b = byte[0] as u32;

            val |= (b & PART) << (size * 7);
            size += 1;
            if size > 5 {
                // return Err(Error::Err("VarInt too big".to_owned()));
                return Err(CodecError::VarInt(VarIntError::TooBig));
            }
            if (b & 0x80) == 0 {
                break;
            }
        }

        Ok(VarInt(val as i32))
    }
}

impl Encoder for VarInt {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<usize> {
        let mut x = self.0 as u32;
        let mut i = 0;
        loop {
            let mut temp = (x & 0b0111_1111) as u8;
            x >>= 7;
            if x != 0 {
                temp |= 0b1000_0000;
            }

            writer.write(&[temp])?;

            i += 1;
            if x == 0 {
                break;
            }
        }
        Ok(i)
    }
}

#[test]
fn test_varint_ed() {
    let mut buffer = Vec::new();
    let value = VarInt(0x7F);
    value.encode(&mut buffer).unwrap();
    assert_eq!(buffer, [0x7F]);

    let decoded = VarInt::decode(&mut buffer.as_slice()).unwrap();
    assert_eq!(decoded, value);

    let mut buffer = Vec::new();
    let value = VarInt(0x3FFF);
    value.encode(&mut buffer).unwrap();
    assert_eq!(buffer, [0xFF, 0x7F]);

    let decoded = VarInt::decode(&mut buffer.as_slice()).unwrap();
    assert_eq!(decoded, value);

    let mut buffer = Vec::new();
    let value = VarInt(0x1FFFFF);
    value.encode(&mut buffer).unwrap();
    assert_eq!(buffer, [0xFF, 0xFF, 0x7F]);

    let decoded = VarInt::decode(&mut buffer.as_slice()).unwrap();
    assert_eq!(decoded, value);

    let mut buffer = Vec::new();
    let value = VarInt(0xFFFFFFF);
    value.encode(&mut buffer).unwrap();
    assert_eq!(buffer, [0xFF, 0xFF, 0xFF, 0x7F]);

    let decoded = VarInt::decode(&mut buffer.as_slice()).unwrap();
    assert_eq!(decoded, value);

    let mut buffer = Vec::new();
    let value = VarInt(0x0);
    value.encode(&mut buffer).unwrap();
    assert_eq!(buffer, [0x00]);

    let decoded = VarInt::decode(&mut buffer.as_slice()).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn var_int_de() {
    let mut buffer = Vec::new();
    let value = VarInt(69420);
    value.encode(&mut buffer).unwrap();
    buffer.extend_from_slice(b"hello world");

    let mut buffer = buffer.as_slice();
    let decoded = VarInt::decode(&mut buffer).unwrap();
    assert_eq!(decoded, value);
}
