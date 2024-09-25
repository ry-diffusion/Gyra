use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(CodecDecode, CodecEncode, Debug, Clone, PartialEq)]
#[packet(id: 0x08, when: Play)]
pub struct PlayerPositionAndLook {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: u8,
}

#[derive(CodecDecode, CodecEncode, Debug, Clone, PartialEq)]
#[packet(id: 0x04, when: Play, server)]
pub struct PlayerPosition {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub on_ground: bool,
}
