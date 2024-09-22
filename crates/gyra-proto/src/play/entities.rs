use gyra_codec::variadic_int::VarInt;
use gyra_macros::{CodecDecode, CodecEncode, packet};

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x14, when: Play)]
pub struct Entity {
    pub entity_id: VarInt,
}

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x15, when: Play)]
pub struct EntityRelativeMove {
    pub entity_id: VarInt,
    pub delta_x: i8,
    pub delta_y: i8,
    pub delta_z: i8,
    pub on_ground: bool,
}
