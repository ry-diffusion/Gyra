use gyra_codec::variadic_int::VarInt;
use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x03, when: Login)]
pub struct SetCompression {
    pub threshold: VarInt,
}
