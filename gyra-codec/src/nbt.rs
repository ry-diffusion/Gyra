use crate::coding::{Decoder, Encoder};

#[derive(Debug, Clone, PartialEq)]
pub struct Nbt<T: serde::Serialize + serde::de::DeserializeOwned> {
    pub value: T,
}

impl<T: serde::Serialize + serde::de::DeserializeOwned> Decoder for Nbt<T> {
    fn decode<R: std::io::Read>(reader: &mut R) -> crate::error::Result<Self> {
        let value = fastnbt::from_reader(reader)?;
        Ok(Nbt { value })
    }
}

impl<T: serde::Serialize + serde::de::DeserializeOwned> Encoder for Nbt<T> {
    fn encode<W: std::io::Write>(&self, writer: &mut W) -> crate::error::Result<usize> {
        let bytes = fastnbt::to_bytes(&self.value)?;

        writer.write_all(&bytes)?;

        Ok(bytes.len())
    }
}
