use crate::coding::Decoder;

#[derive(Clone, Debug, PartialEq)]
pub struct NibbleArray {
    data: Vec<u8>,
}

impl NibbleArray {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn get(&self, index: usize) -> u8 {
        let byte = self.data[index / 2];
        if index % 2 == 0 {
            byte & 0x0F
        } else {
            byte >> 4
        }
    }
    
    pub fn set(&mut self, index: usize, value: u8) {
        let byte = &mut self.data[index / 2];
        if index % 2 == 0 {
            *byte = (*byte & 0xF0) | (value & 0x0F);
        } else {
            *byte = (*byte & 0x0F) | ((value & 0x0F) << 4);
        }
    }
    
    pub fn size(&self) -> usize {
        self.data.len() * 2
    }
}
