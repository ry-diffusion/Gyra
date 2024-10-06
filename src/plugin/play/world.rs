use crate::plugin::play::player;
use bevy::math::U16Vec2;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use gyra_proto::distance::ChunkVec2;
use gyra_proto::smp;

#[derive(Resource, Default)]
pub struct ActivePlayerChunks {
    pub chunks: HashMap<ChunkVec2, smp::ChunkColumn>,
}

#[derive(Resource, Default)]
pub struct ShownPlayerChunks {
    pub renderized: HashSet<ChunkVec2>,
    pub in_front: HashSet<ChunkVec2>,
    pub behind: HashSet<ChunkVec2>,
}

#[derive(Component)]
/// this is a mark when the entity is on ground.
pub struct OnGround;

#[derive(Resource)]
pub struct ChunkLoadDistance(pub u32);

#[derive(Component)]
pub struct Block;

#[derive(Resource, Default)]
pub struct WorldChunkData {
    // X-Z -> Section
    pub loaded_column: HashMap<ChunkVec2, smp::ChunkColumn>,
}

pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(WorldChunkData::default())
        .insert_resource(ActivePlayerChunks::default())
        .insert_resource(ShownPlayerChunks::default())
        /* 16 chunk column */
        /* Make this memory dependent? Like use 1/2 of memory for rendering... */
        .insert_resource(ChunkLoadDistance(2))
        .add_systems(Update, update_active_chunks);
}

fn is_chunk_within_view_distance(
    chunk_distance: &ChunkVec2,
    player_chunk_distance: &ChunkVec2,
    view_distance: i32,
) -> bool {
    let distance_x = (chunk_distance.x - player_chunk_distance.x).abs();
    let distance_z = (chunk_distance.z - player_chunk_distance.z).abs();

    distance_x <= view_distance && distance_z <= view_distance
}

fn update_active_chunks(
    player_q: Query<&Transform, (With<player::Player>, Changed<Transform>)>,
    mut active: ResMut<ActivePlayerChunks>,
    world_data: Res<WorldChunkData>,
    view_distance: Res<ChunkLoadDistance>,
    mut old_distance: Local<IVec3>,
) {
    if let Ok(player) = player_q.get_single() {
        // Hey Google Translator!
        let translation = player.translation.as_ivec3();

        if translation.x == old_distance.x
            && translation.z == old_distance.z
            && active.chunks.len() > 1
            && !world_data.is_changed()
        {
            return;
        }

        *old_distance = translation.clone();

        info!("Updating active chunks");

        let mut columns = HashMap::new();
        let player_distance = ChunkVec2::new_global(translation.x, translation.z);

        for (chunk_pos, chunk) in &world_data.loaded_column {
            if is_chunk_within_view_distance(chunk_pos, &player_distance, view_distance.0 as _) {
                columns.insert(*chunk_pos, chunk.clone());
            }
        }

        active.chunks = columns;
    }
}
