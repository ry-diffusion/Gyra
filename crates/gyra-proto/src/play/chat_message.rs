use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(Clone, Debug, CodecEncode, CodecDecode, PartialEq)]
#[packet(id: 0x02, when: Play)]
pub struct ChatMessage {
    pub content: String,
    pub position: u8,
}

#[derive(Clone, Debug, CodecEncode, CodecDecode, PartialEq)]
#[packet(id: 0x01, when: Play, server)]
pub struct SendChatMessage {
    pub content: String,
}
