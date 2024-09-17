use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(Debug, Clone, CodecDecode, CodecEncode, PartialEq)]
#[packet(id: 0x01, when: Status)]
pub struct PingPong {
    pub payload: i64,
}

impl PingPong {
    pub fn new(payload: i64) -> Self {
        Self { payload }
    }
    pub fn now() -> Self {
        let payload = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        Self { payload }
    }
}