use gyra_macros::{CodecDecode, CodecEncode, packet};

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x1F, when: Play, client)]
pub struct ClientKeepAlive {
    pub id: i64,
}

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x10, when: Play, server)]
pub struct ServerKeepAlive {
    pub id: i64,
}
