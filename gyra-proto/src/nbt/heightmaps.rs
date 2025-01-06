use fastnbt::LongArray;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
/// Compound containing one long array named MOTION_BLOCKING, which is a heightmap for the highest solid block at each position in the chunk (as a compacted long array with 256 entries at 9 bits per entry totaling 36 longs). The Notchian server also adds a WORLD_SURFACE long array, the purpose of which is unknown, but it's not required for the chunk to be accepted.
pub struct Heightmaps {
    #[serde(rename = "MOTION_BLOCKING")]
    pub motion_blocking: LongArray,

    #[serde(rename = "WORLD_SURFACE")]
    pub world_surface: Option<LongArray>,
}
