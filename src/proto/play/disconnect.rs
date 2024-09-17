use gyra_macros::{CodecDecode, CodecEncode, packet};

#[derive(Clone, Debug, CodecEncode, CodecDecode, PartialEq)]
#[packet(id: 0x40, when: Play)]
pub struct Disconnect {
    pub reason: String,
}