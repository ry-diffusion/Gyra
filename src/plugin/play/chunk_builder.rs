use bevy::prelude::*;
use gyra_proto::smp;
use std::collections::HashMap;

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
        .add_systems(Startup, load_materials)
        .add_systems(Update, spawn_chunks);
}

fn load_materials(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let dirt = materials.add(Color::srgb_u8(138, 69, 58));
    let grass = materials.add(Color::srgb_u8(1, 200, 0));
    let endstone_material = materials.add(Color::srgb_u8(216, 214, 164));
    let netherbrick_material = materials.add(Color::srgb_u8(63, 42, 35));
    let any_block = materials.add(Color::srgb_u8(0, 0, 0));
    let bedrock = materials.add(Color::srgb_u8(0, 0, 0));
    let mut blocks = HashMap::new();

    blocks.insert(121, endstone_material);
    blocks.insert(112, netherbrick_material);
    blocks.insert(2, grass);
    blocks.insert(3, dirt);
    blocks.insert(0, bedrock);

    commands.insert_resource(Materials { blocks, any_block });
}

pub fn spawn_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: Res<Materials>, 
    mut events: EventReader<ChunkReceived>,
    
) {
    for chunk in events.read() {
        let chunk = &chunk.smp_chunk;
        let ((start_x, start_y, start_z), (end_x, end_y, end_z)) = chunk.get_world_coordinates();
        info!("Building chunks of {start_x} {start_y} {start_z} to {end_x}, {end_y}, {end_z}");
        let mut bundles = vec![];
        let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    if let Some(block_id) = chunk.block_id_of(x, y, z) {
                        if 0 != block_id {
                            let (x, y, z) = chunk.block_coordinates(x, y, z);

                            bundles.push(MaterialMeshBundle {
                                mesh: mesh.clone(),
                                material: materials.get_material_by_block_id(block_id as _), 
                                transform: Transform::from_xyz(x as _, y as _, z as _),
                                ..default()
                            });
                            // bundles.push(PbrBundle {

                            //     material: materials.add(block_color),
                            //     transform: Transform::from_xyz(x as _, y as _, z as _),
                            //
                            //     ..default()
                            // });
                        }
                    }
                }
            }
        }

        if bundles.len() > 100 {
            warn!("Too many bundles!");
        }

        info!("Loading {} bundles...", bundles.len());
        commands.spawn_batch(bundles);
    }
}
