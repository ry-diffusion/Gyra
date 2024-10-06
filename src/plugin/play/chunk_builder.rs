use std::time::Instant;

use crate::plugin::play::player::Player;
use crate::plugin::play::world::{ActivePlayerChunks, ShownPlayerChunks, WorldChunkData};
use crate::state::AppState;
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Face, PrimitiveTopology};
use bevy::utils::{HashMap, HashSet};
use gyra_proto::distance::ChunkVec2;
use gyra_proto::smp;

use super::chunk_cons::ChunkConstructor;

#[derive(Event, Debug)]
pub struct RenderChunk {
    pub pos: ChunkVec2,
}

#[derive(Event, Debug)]
pub struct UnrenderChunk {
    pub pos: ChunkVec2,
}

#[derive(Event, Debug)]
pub struct ChunkReceived {
    pub smp_chunk: smp::ChunkColumn,
}

#[derive(Resource)]
pub struct Materials {
    pub blocks: HashMap<u32, Handle<StandardMaterial>>,
    pub any_block: Handle<StandardMaterial>,
}

impl Materials {
    pub fn get_material_by_block_id(&self, id: u32) -> Handle<StandardMaterial> {
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
        .add_systems(
            PreUpdate,
            (download_chunks, chunk_scheduler).run_if(in_state(AppState::Playing)),
        )
        .add_systems(Update, (render_chunks).run_if(in_state(AppState::Playing)))
        .add_systems(
            PostUpdate,
            (unrender_chunks).run_if(in_state(AppState::Playing)),
        )
        .add_systems(Startup, load_materials);
}

fn build_material_by_color(base_color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color,

        double_sided: true,
        cull_mode: Some(Face::Back),
        ..default()
    }
}

fn load_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let dirt = materials.add(build_material_by_color(Color::srgb_u8(138, 69, 58)));
    let grass = materials.add(build_material_by_color(Color::srgb_u8(1, 200, 0)));
    let endstone_material = materials.add(build_material_by_color(Color::srgb_u8(216, 214, 164)));
    let netherbrick_material = materials.add(build_material_by_color(Color::srgb_u8(63, 42, 35)));
    let any_block = materials.add(build_material_by_color(Color::srgb_u8(0, 0, 0)));
    let bedrock = materials.add(build_material_by_color(Color::srgb_u8(0, 0, 0)));

    let mut blocks = HashMap::new();

    blocks.insert(121, endstone_material);
    blocks.insert(112, netherbrick_material);
    blocks.insert(2, grass);
    blocks.insert(3, dirt);
    blocks.insert(0, bedrock);

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
            if is_chunk_in_front(player.translation, forward_dir, *dist_chunk, pespective.fov) {
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

fn build_cube_mesh() -> Mesh {
    let vertices = vec![
        // Front
        [-0.5, -0.5, 0.5], // BL
        [0.5, -0.5, 0.5],  // BR
        [0.5, 0.5, 0.5],   // TR
        [-0.5, 0.5, 0.5],  // TL
        // Back
        [-0.5, -0.5, -0.5], // BL
        [0.5, -0.5, -0.5],  // BR
        [0.5, 0.5, -0.5],   // TR
        [-0.5, 0.5, -0.5],  // TL
    ];

    // let positions = vertices.into_iter().flat_map(|x| x).collect::<Vec<f32>>();

    let indices = Indices::U32(vec![
        // Front
        0, 1, 2, 2, 3, 0, // Right
        1, 5, 6, 6, 2, 1, // Back
        7, 6, 5, 5, 4, 7, // Left
        4, 0, 3, 3, 7, 4, // Bottom
        4, 5, 1, 1, 0, 4, // Top
        3, 2, 6, 6, 7, 3,
    ]);

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(indices);

    mesh
}

fn unrender_chunks(
    mut commands: Commands,
    loaded_q: Query<(Entity, &ParentChunk)>,
    mut to_unrender: EventReader<UnrenderChunk>,
) {
    for chunk_pos in to_unrender.read() {
        for (entity, parent) in loaded_q.iter() {
            if parent.of == chunk_pos.pos {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn render_chunks(
    active_chunks: Res<ActivePlayerChunks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    materials_pre: Res<Materials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut to_render: EventReader<RenderChunk>,
) {
    // let material = materials.add(StandardMaterial {
    //     base_color: Color::srgba(0.5, 0.8, 0.5, 0.5),
    //     alpha_mode: AlphaMode::Blend,
    //     ..Default::default()
    // });
    for (chunk_pos, _) in to_render.par_read() {
        let column = active_chunks.chunks.get(&chunk_pos.pos).unwrap();
        let mut chunk_cons = ChunkConstructor::new(column);

        let now = Instant::now();
        let mut to_spawn = vec![];

        for (mesh_recipe, transform) in chunk_cons.construct() {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            );

            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_recipe.vertices);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_recipe.normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_recipe.uv);
            mesh.insert_indices(Indices::U32(mesh_recipe.indices));

            to_spawn.push((
                PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials_pre.get_material_by_block_id(2),
                    transform,
                    ..Default::default()
                },
                ParentChunk { of: chunk_pos.pos },
            ));
        }

        commands.spawn_batch(to_spawn);

        info!("Chunk spawn time: {:?}", now.elapsed());
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
