use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(Clone, Debug, CodecEncode, CodecDecode, PartialEq)]
#[packet(id: 0x02, when: Play)]
pub struct ChatMessage {
    pub content: String,
    pub position: u8,
}