use std::io::Read;

use crate::{
    coding::{Decoder, Encoder},
    variadic_int::VarInt,
};
#[derive(Debug, Clone, PartialEq)]
pub struct Array<T: Decoder + Encoder> {
    pub elements: Vec<T>,
}

impl<T: Encoder + Decoder> Decoder for Array<T> {
    fn decode<R: Read>(reader: &mut R) -> crate::error::Result<Self> {
        let len = VarInt::decode(reader)?.0;

        let mut elements = Vec::with_capacity(len as usize);

        for _ in 0..len {
            elements.push(T::decode(reader)?);
        }

        Ok(Array { elements })
    }
}

impl<T: Encoder + Decoder> Encoder for Array<T> {
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> crate::error::Result<usize> {
        let len = VarInt(self.elements.len() as i32);
        let mut bytes = len.encode(writer)?;

        for element in &self.elements {
            bytes += element.encode(writer)?;
        }

        Ok(bytes)
    }
}
