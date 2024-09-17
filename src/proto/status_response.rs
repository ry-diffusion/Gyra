#[derive(Debug, Clone, gyra_macros::CodecDecode, gyra_macros::CodecEncode, PartialEq)]
#[gyra_macros::packet(id: 0x00, when: Status)]
pub struct StatusResponse {
    pub json_response: String,
}