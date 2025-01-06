use gyra_codec::array::Array;
use gyra_macros::{CodecDecode, CodecEncode, packet};

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x17, when: Play, client)]
pub struct ClientPluginMessage {
    pub channel: String,
    pub data: Array<u8>,
}
