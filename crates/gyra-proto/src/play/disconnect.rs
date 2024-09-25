use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(Clone, Debug, CodecEncode, CodecDecode, PartialEq)]
#[packet(id: 0x40, when: Play)]
pub struct Disconnect {
    pub reason: String,
}
