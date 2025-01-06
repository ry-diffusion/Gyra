use gyra_codec::variadic_int::VarInt;
use gyra_macros::{CodecDecode, CodecEncode, packet};

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x34, when: Play, client)]
pub struct PlayerPositionAndLook {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: u8,
    pub teleport_id: VarInt,
}
