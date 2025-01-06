use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DimensionValues {
    pub piglin_safe: bool,
    pub natural: bool,
    pub ambient_light: f32,
    pub infiniburn: String,
    pub respawn_anchor_works: bool,
    pub has_skylight: bool,
    pub bed_works: bool,
    pub has_raids: bool,
    pub logical_height: i32,
    pub coordinate_scale: f64,
    pub ultrawarm: bool,
    pub has_ceiling: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DimensionValue {
    pub name: String,
    pub id: i32,
    pub element: DimensionValues,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BiomeEffects {
    pub sky_color: i32,
    pub water_fog_color: i32,
    pub fog_color: i32,
    pub water_color: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BiomeElement {
    pub downfall: f32,
    pub temperature: f32,
    pub scale: f32,
    pub effects: BiomeEffects,
    pub precipitation: String,
    pub depth: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BiomeType {
    pub id: i32,
    pub element: BiomeElement,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct DimensionRegistry {
    #[serde(rename = "type")]
    pub type_name: String,
    pub value: Vec<DimensionValue>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct BiomeRegistry {
    pub value: Vec<BiomeType>,

    #[serde(rename = "type")]
    pub type_name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Registry {
    #[serde(rename = "minecraft:dimension_type")]
    pub dimensions: DimensionRegistry,
    #[serde(rename = "minecraft:worldgen/biome")]
    pub biomes: BiomeRegistry,
}
