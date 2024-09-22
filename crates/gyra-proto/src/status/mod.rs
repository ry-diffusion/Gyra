use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(Debug, Clone, CodecDecode, CodecEncode, PartialEq)]
#[packet(id: 0x01, when: Status)]
pub struct PingPong {
    pub payload: i64,
}

impl PingPong {
    pub fn now() -> Self {
        let payload = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Self { payload }
    }
}

#[derive(Debug, Clone, gyra_macros::CodecDecode, gyra_macros::CodecEncode, PartialEq)]
#[packet(id: 0x00, when: Status)]
pub struct StatusRequest;


#[derive(Debug, Clone, gyra_macros::CodecDecode, gyra_macros::CodecEncode, PartialEq)]
#[packet(id: 0x00, when: Status)]
pub struct StatusResponse {
    pub json_response: String,
}
