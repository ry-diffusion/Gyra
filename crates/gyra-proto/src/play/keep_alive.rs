use gyra_codec::variadic_int::VarInt;
use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(Clone, Debug, CodecEncode, CodecDecode, PartialEq)]
#[packet(id: 0x00, when: Play)]
pub struct KeepAlive {
    pub id: VarInt,
}
