use super::chunk_cons::ChunkConstructor;
use crate::plugin::consts::WorldLayer;
use crate::plugin::play::player::Player;
use crate::plugin::play::world::{ActivePlayerChunks, ShownPlayerChunks, WorldChunkData};
use crate::state::AppState;
use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use bevy::utils::{HashMap, HashSet};
use gyra_proto::distance::ChunkVec2;
use gyra_proto::smp;

#[derive(Event, Debug)]
pub struct RenderChunk {
    pub pos: ChunkVec2,
}

#[derive(Event, Debug)]
pub struct UnrenderChunk {
    pub pos: ChunkVec2,
}

#[derive(Event, Debug, Clone)]
pub struct RenderedBlock {
    pub mesh: Mesh,
    pub material_id: u16,
    pub transform: Transform,
    pub parent_chunk: ChunkVec2,
}

#[derive(Event, Debug)]
pub struct ChunkReceived {
    pub smp_chunk: smp::ChunkColumn,
}

#[derive(Resource)]
pub struct Materials {
    pub blocks: HashMap<u16, Handle<StandardMaterial>>,
    pub any_block: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct ChunkBuilderTasks {
    pub tasks: Vec<Task<Vec<RenderedBlock>>>,
}

impl Materials {
    pub fn get_material_by_block_id(&self, id: u16) -> Handle<StandardMaterial> {
        let block = self.blocks.get(&id);
        if let Some(block) = block {
            block.clone_weak()
        } else {
            self.any_block.clone_weak()
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_event::<ChunkReceived>()
        .add_event::<RenderChunk>()
        .add_event::<UnrenderChunk>()
        .add_event::<RenderedBlock>()
        .insert_resource(ChunkBuilderTasks { tasks: vec![] })
        .add_systems(
            PreUpdate,
            (download_chunks, chunk_scheduler).run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            (process_chunks, render_chunks).run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            PostUpdate,
            unrender_chunks.run_if(in_state(AppState::Playing)),
        )
        .add_systems(Startup, load_materials)
        .add_systems(OnExit(AppState::Playing), cleanup_chunks);
}

fn cleanup_chunks(
    mut commands: Commands,
    mut active_chunks: ResMut<ActivePlayerChunks>,
    loaded_q: Query<(Entity, &ParentChunk)>,
    mut shown: ResMut<ShownPlayerChunks>,
    mut world_data: ResMut<WorldChunkData>,
) {
    for (entity, _) in loaded_q.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // bye :c
    active_chunks.chunks.clear();
    world_data.loaded_column.clear();
    shown.renderized.clear();
}

fn build_material_by_color(base_color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color,

        alpha_mode: AlphaMode::Blend,
        double_sided: false,
        cull_mode: None,
        ..default()
    }
}

fn load_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let dirt = materials.add(build_material_by_color(Color::srgb_u8(138, 69, 58)));
    let grass = materials.add(build_material_by_color(Color::srgb_u8(1, 200, 0)));
    let endstone_material = materials.add(build_material_by_color(Color::srgb_u8(216, 214, 164)));
    let netherbrick_material = materials.add(build_material_by_color(Color::srgb_u8(63, 42, 35)));
    let any_block = materials.add(build_material_by_color(Color::srgb_u8(255, 255, 255)));
    let bedrock = materials.add(build_material_by_color(Color::srgb_u8(0, 0, 0)));
    let cobblestone = materials.add(build_material_by_color(Color::srgb_u8(150, 150, 150)));
    let water = materials.add(build_material_by_color(Color::srgba_u8(0, 0, 255, 150)));
    let jungle_wood = materials.add(build_material_by_color(Color::srgb_u8(139, 69, 19)));
    let gravel = materials.add(build_material_by_color(Color::srgb_u8(104, 104, 104)));

    let mut blocks = HashMap::new();

    blocks.insert(121, endstone_material);
    blocks.insert(112, netherbrick_material);
    blocks.insert(2, grass);
    blocks.insert(3, dirt);
    blocks.insert(7, bedrock);
    blocks.insert(1, cobblestone);
    blocks.insert(9, water);
    blocks.insert(17, jungle_wood);
    blocks.insert(13, gravel);

    commands.insert_resource(Materials { blocks, any_block });
}

#[derive(Component, Debug)]
struct ParentChunk {
    pub of: ChunkVec2,
}

fn is_chunk_in_front(
    player_pos: Vec3,
    forward_dir: Vec3,
    chunk_pos: ChunkVec2,
    fov_radians: f32,
) -> bool {
    // Calculate the chunk center position
    let chunk_center = Vec3::new(
        chunk_pos.x as f32 * 16.0 + 8.0, // Chunk center X
        player_pos.y,                    // Match Y with camera to simplify calculations
        chunk_pos.z as f32 * 16.0 + 8.0, // Chunk center Z
    );

    // Vector from camera to the chunk
    let to_chunk = chunk_center - player_pos;

    // Normalize the vector from camera to chunk
    let to_chunk_normalized = to_chunk.normalize();

    // Calculate the dot product between the forward direction and the vector to the chunk
    let dot_product = forward_dir.dot(to_chunk_normalized);
    let cos_fov = fov_radians.cos();

    // If the dot product is greater than the cosine of the FOV, the chunk is in front
    dot_product > cos_fov
}

fn chunk_scheduler(
    active: Res<ActivePlayerChunks>,
    mut shown: ResMut<ShownPlayerChunks>,
    player_q: Query<&Transform, (With<Player>, Changed<Transform>)>,
    cam_q: Query<
        &Projection,
        (
            With<crate::plugin::play::player::WorldModelCamera>,
            Without<Player>,
        ),
    >,

    mut render_writer: EventWriter<RenderChunk>,
    mut unrender_writer: EventWriter<UnrenderChunk>,
) {
    let projection = cam_q.single();

    let Projection::Perspective(pespective) = projection else {
        unreachable!("No perspective projection found");
    };

    let mut to_render_count = 0;
    let mut to_unrender_count = 0;

    // The part of the frustum that is in front of the player
    let mut frustum = HashSet::new();
    if let Ok(player) = player_q.get_single() {
        let forward_dir = *player.forward();

        for dist_chunk in active.chunks.keys() {
            if is_chunk_in_front(
                player.translation,
                forward_dir,
                *dist_chunk,
                pespective.fov * 2.0,
            ) {
                frustum.insert(*dist_chunk);
            }
        }
    }

    let mut frustum_to_render = vec![];
    for chunk in frustum.iter() {
        if !shown.renderized.contains(chunk) {
            frustum_to_render.push(RenderChunk { pos: *chunk });
            to_render_count += 1;
        }
    }

    render_writer.send_batch(frustum_to_render);

    let mut culled_frustum = vec![];

    for chunk in shown.renderized.iter() {
        if !frustum.contains(chunk) {
            unrender_writer.send(UnrenderChunk { pos: *chunk });
            to_unrender_count += 1;
            culled_frustum.push(*chunk);
        }
    }

    shown.renderized.extend(frustum);

    for chunk in culled_frustum.iter() {
        shown.renderized.remove(chunk);
    }

    if to_render_count > 0 || to_unrender_count > 0 {
        info!("To render: {to_render_count}, to unrender: {to_unrender_count}");
    }
}

fn unrender_chunks(
    mut commands: Commands,
    loaded_q: Query<(Entity, &ParentChunk)>,
    mut to_unrender: EventReader<UnrenderChunk>,
) {
    for (chunk_pos, _) in to_unrender.par_read() {
        for (entity, parent) in loaded_q.iter() {
            if parent.of == chunk_pos.pos {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn process_chunks(
    mut rendered_writer: EventWriter<RenderedBlock>,
    active_player_chunks: Res<ActivePlayerChunks>,
    mut to_render: EventReader<RenderChunk>,
    mut tasks: ResMut<ChunkBuilderTasks>,
) {
    // Lets generate mesh for chunks in async compute poll

    let poll = AsyncComputeTaskPool::get();

    let build_neighbors = |pos: ChunkVec2| {
        let mut neighbors = HashMap::<IVec3, smp::ChunkColumn>::new();

        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for (x, z) in directions.iter() {
            let neighbor_pos = IVec3::new(pos.x + x, 0, pos.z + z);
            let chpos = ChunkVec2::new_local(neighbor_pos.x, neighbor_pos.z);
            if let Some(neighbor) = active_player_chunks.chunks.get(&chpos) {
                neighbors.insert(neighbor_pos, neighbor.clone());
            }
        }

        neighbors
    };

    for (pos, _) in to_render.par_read() {
        if let Some(column) = active_player_chunks.chunks.get(&pos.pos) {
            let column = column.clone();
            let parent_chunk = pos.pos;
            let neighbors = build_neighbors(parent_chunk);

            let task = poll.spawn(async move {
                let mut constructor = ChunkConstructor::new(&column, neighbors);
                let result = constructor.construct(Vec3::ZERO);

                let mut to_send = vec![];

                for (mesh_recipe, transform, id) in result {
                    let mut mesh = Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::RENDER_WORLD,
                    );

                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_recipe.vertices);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_recipe.normals);
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_recipe.uv);
                    mesh.insert_indices(Indices::U32(mesh_recipe.indices));

                    to_send.push(RenderedBlock {
                        mesh,
                        material_id: id as _,
                        transform,
                        parent_chunk,
                    });
                }

                to_send
            });
            tasks.tasks.push(task);
        }
    }

    let mut to_remove = vec![];
    let mut rendered = vec![];
    for (idx, task) in tasks.tasks.iter_mut().enumerate() {
        let status = if task.is_finished() {
            Some(block_on(task))
        } else {
            block_on(poll_once(task))
        };

        match status {
            Some(res) => {
                rendered.extend(res);

                to_remove.push(idx);
            }

            None => {}
        }
    }

    for idx in to_remove.iter().rev() {
        let _ = tasks.tasks.remove(*idx);
    }

    rendered_writer.send_batch(rendered);
}

fn render_chunks(
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    materials_pre: Res<Materials>,
    mut to_render: EventReader<RenderedBlock>,
) {
    let mut to_spawn = vec![];

    for (block, _) in to_render.par_read() {
        let block = block.to_owned();

        to_spawn.push((
            MaterialMeshBundle {
                mesh: meshes.add(block.mesh),
                material: materials_pre.get_material_by_block_id(block.material_id),
                transform: block.transform,
                ..Default::default()
            },
            WorldLayer,
            ParentChunk {
                of: block.parent_chunk,
            },
        ));
    }

    if !to_spawn.is_empty() {
        info!("Rendering {}", to_spawn.len());
        commands.spawn_batch(to_spawn);
    }
}

fn download_chunks(
    mut chunks_received: EventReader<ChunkReceived>,
    mut chunk_data: ResMut<WorldChunkData>,
) {
    for chunk_pkt in chunks_received.read() {
        let column = &chunk_pkt.smp_chunk;

        chunk_data
            .loaded_column
            .insert(ChunkVec2::new_local(column.x, column.z), column.clone());

        debug!(
            "Total chunks loaded until now: {}",
            chunk_data.loaded_column.keys().count()
        );
    }
}
