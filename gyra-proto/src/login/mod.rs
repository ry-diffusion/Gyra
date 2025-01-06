use gyra_codec::{uuid::Uuid, variadic_int::VarInt};
use gyra_macros::{CodecDecode, CodecEncode, packet};

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x00, when: Login, server)]
pub struct LoginStart {
    pub username: String,
}

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x02, when: Login)]
pub struct LoginSuccess {
    pub uuid: Uuid,
    pub username: String,
}

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x03, when: Login)]
pub struct SetCompression {
    pub threshold: VarInt,
}
