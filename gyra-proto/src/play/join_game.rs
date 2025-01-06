use gyra_codec::{array::Array, nbt::Nbt, variadic_int::VarInt};
use gyra_macros::{CodecDecode, CodecEncode, packet};

use crate::nbt::dimension::{DimensionRegistry, DimensionValue, DimensionValues, Registry};

#[derive(CodecDecode, CodecEncode, Debug, PartialEq)]
#[packet(id: 0x24, when: Play, client)]
pub struct JoinGame {
    /// The player's Entity ID (EID).
    pub entity_id: i32,
    /// If true, the server is in hardcore mode.
    pub is_hardcore: bool,
    /// 0: Survival, 1: Creative, 2: Adventure, 3: Spectato
    pub gamemode: u8,
    /// 0: survival, 1: creative, 2: adventure, 3: spectator. The hardcore flag is not included. The previous gamemode. Defaults to -1 if there is no previous gamemode. (More information needed)
    pub previous_gamemode: i8,
    /* pub: world_count: VarInt */
    /// Identifiers for all worlds on the server.
    pub world_names: Array<String>,
    /// The full extent of these is still unknown, but the tag represents a dimension and biome registry. See below for the vanilla default.
    pub dimension_codec: Nbt<Registry>,
    /// Valid dimensions are defined per dimension registry sent before this. The structure of this tag is a dimension type (see below).
    pub dimension: Nbt<DimensionValues>,
    /// The name of the world being spawned into.
    pub world_name: String,
    /// First 8 bytes of the SHA-256 hash of the world's seed. Used client side for biome noise
    pub hashed_seed: i64,
    /// Was once used by the client to draw the player list, but now is ignored.
    pub max_players: VarInt,
    /// Render distance (2-32).
    pub view_distance: VarInt,
    /// If true, a Notchian client shows reduced information on the debug screen. For servers in development, this should almost always be false.
    pub reduced_debug_info: bool,
    /// Set to false when the doImmediateRespawn gamerule is true.
    pub enable_respawn_screen: bool,
    /// True if the world is a debug mode world; debug mode worlds cannot be modified and have predefined blocks.
    pub is_debug: bool,
    /// True if the world is a superflat world; flat worlds have different void fog and a horizon at y=0 instead of y=63.
    pub is_flat: bool,
}
