use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x00, when: Login)]
pub struct LoginStart {
    pub username: String,
}
