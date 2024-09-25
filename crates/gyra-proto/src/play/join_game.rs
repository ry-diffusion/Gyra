use gyra_macros::{packet, CodecDecode, CodecEncode};

#[derive(Clone, Debug, CodecEncode, CodecDecode, PartialEq)]
#[packet(id: 0x01, when: Play)]
pub struct JoinGame {
    pub entity_id: i32,
    pub game_mode: u8,
    pub dimension: i8,
    pub difficulty: u8,
    pub max_players: u8,
    pub level_type: String,
    pub reduced_debug_info: bool,
}
