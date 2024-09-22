use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x02, when: Login)]
pub struct LoginSuccess {
    pub uuid: String,
    pub username: String,
}
