use bevy::{
    log::info,
    math::{IVec3, Vec3},
    pbr::StandardMaterial,
    prelude::Transform,
    utils::HashMap,
};
use gyra_proto::smp;
use std::hash::Hash;

use super::block_builder::Block;

#[derive(Default, Debug, Clone)]
pub struct BlockMesh {
    // vertices
    pub vertices: Vec<[f32; 3]>,
    // lighting normals
    pub normals: Vec<[f32; 3]>,
    // uv coordinates
    pub uv: Vec<[f32; 2]>,
    // indices
    pub indices: Vec<u32>,
}

pub struct ChunkConstructor<'a> {
    pub column: &'a smp::ChunkColumn,
    pub pos: IVec3,
    pub neighbors: HashMap<IVec3, smp::ChunkColumn>,
}

impl<'a> ChunkConstructor<'a> {
    pub fn new(column: &'a smp::ChunkColumn, neighbors: HashMap<IVec3, smp::ChunkColumn>) -> Self {
        Self {
            column,
            pos: IVec3::new(column.x, 0, column.z),
            neighbors,
        }
    }

    #[inline]
    fn block_at(&self, section: usize, pos: IVec3) -> Option<Block> {
        if pos.x >= 0 && pos.x < 16 && pos.y >= 0 && pos.y < 16 && pos.z >= 0 && pos.z < 16 {
            let column_section = self.column.sections[section].as_ref()?;
            let id = column_section.block_id(pos.x as _, pos.y as _, pos.z as _);
            Some(Block::from_id(id))
        } else {
            let chunk_offset = IVec3::new(
                if pos.x < 0 {
                    -1
                } else if pos.x >= 16 {
                    1
                } else {
                    0
                },
                0,
                if pos.z < 0 {
                    -1
                } else if pos.z >= 16 {
                    1
                } else {
                    0
                },
            );
            let neighbor_pos = self.pos + chunk_offset;

            if let Some(neighbor) = self.neighbors.get(&neighbor_pos) {
                let neighbor_section = neighbor.sections[section].as_ref()?;
                let neighbor_pos = pos - chunk_offset * 16;
                let id = neighbor_section.block_id(
                    neighbor_pos.x as _,
                    neighbor_pos.y as _,
                    neighbor_pos.z as _,
                );
                Some(Block::from_id(id))
            } else {
                None
            }
        }
    }

    fn build_block_mesh(&self, block: &Block, pos: IVec3, section: usize) -> BlockMesh {
        if !block.shape().is_solid() {
            return BlockMesh::default();
        }

        let mut vertices = Vec::with_capacity(24);
        let mut normals = Vec::with_capacity(24);
        let mut uv = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        const NATURAL_UV: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        let base_index = vertices.len() as u32;

        let directions = [
            // Front
            (
                IVec3::new(0, 0, 1),
                [0.0, 0.0, 1.0],
                [
                    [0.0, 0.0, 1.0],
                    [1.0, 0.0, 1.0],
                    [1.0, 1.0, 1.0],
                    [0.0, 1.0, 1.0],
                ],
            ),
            // Back
            (
                IVec3::new(0, 0, -1),
                [0.0, 0.0, -1.0],
                [
                    [1.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [1.0, 1.0, 0.0],
                ],
            ),

            // Top
            (
                IVec3::new(0, 1, 0),
                [0.0, 1.0, 0.0],
                [
                    [0.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ],
            ),

            // Bottom
            (
                IVec3::new(0, -1, 0),
                [0.0, -1.0, 0.0],
                [
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 0.0, 1.0],
                    [0.0, 0.0, 1.0],
                ],
            ),

            // Left
            (
                IVec3::new(-1, 0, 0),
                [-1.0, 0.0, 0.0],
                [
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 1.0, 1.0],
                    [0.0, 1.0, 0.0],
                ],
            ),

            // Right
            (
                IVec3::new(1, 0, 0),
                [1.0, 0.0, 0.0],
                [
                    [1.0, 0.0, 1.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [1.0, 1.0, 1.0],
                ],
            ),
        ];

        for (offset, normal, face_vertices) in directions.iter() {
            let idx = base_index + vertices.len() as u32;

            let mut add_face = |face_vertices: &[[f32; 3]],
                                normal: [f32; 3],
                                uv_coords: &[[f32; 2]],
                                base_index: u32| {
                for &vertex in face_vertices {
                    vertices.push(vertex);
                    normals.push(normal);
                }

                for &uv_coord in uv_coords {
                    uv.push(uv_coord);
                }

                indices.extend_from_slice(&[
                    base_index,
                    base_index + 1,
                    base_index + 2,
                    //----
                    base_index + 2,
                    base_index + 3,
                    base_index,
                ]);
            };

            let adjacent_pos = pos + *offset;

            if let Some(adjacent_block) = self.block_at(section, adjacent_pos) {
                if adjacent_block.shape().is_solid() {
                    continue;
                }
            }

            add_face(face_vertices, *normal, &NATURAL_UV, idx);
        }

        BlockMesh {
            vertices,
            normals,
            uv,
            indices,
        }
    }

    pub fn construct(&mut self) -> Vec<(BlockMesh, Transform, u16)> {
        let mut meshes = Vec::new();

        let cull_directions = vec![
            IVec3::new(0, 0, 1),  // Front
            IVec3::new(0, 0, -1), // Back
            IVec3::new(0, 1, 0),  // Top
            IVec3::new(0, -1, 0), // Bottom
            IVec3::new(-1, 0, 0), // Left
            IVec3::new(1, 0, 0),  // Right
        ];

        for (idx, section) in self
            .column
            .sections
            .iter()
            .enumerate()
            .filter_map(|(idx, section)| Some((idx, section.as_ref()?)))
        {
            info!("Rendering section: {idx}");

            let blocks = (0..16).flat_map(|x| {
                (0..16).flat_map(move |y| {
                    (0..16).map(move |z| {
                        let pos = IVec3::new(x, y, z);
                        let block_id = section.block_id(x as _, y as _, z as _);
                        let block = Block::from_id(block_id);

                        (pos, block, block_id)
                    })
                })
            });

            let section_pos = self.pos.as_vec3().with_y(idx as _) * 16.0;

            let blocks = blocks
                .into_iter()
                .filter(|(_, block, _)| block.shape().is_visible());

            // The check
            /*
             * considering this, we only need to render the y blocks, not the x blocks
             yyyy  we only need to render the y blocks
             yxxx  not the x blocks
             yxxx
             yxxx
            */

            let mut edge = HashMap::new();

            for (block_pos, block, id) in blocks {
                let mut has_adjacent_air = false;
                let mut has_adjacent_solid = false;

                for direction in cull_directions.iter() {
                    // we need to check if there is a air adjacent to the block
                    let adjacent_pos = block_pos + *direction;

                    if let Some(adjacent_block) = self.block_at(idx, adjacent_pos) {
                        if adjacent_block.shape().is_visible() {
                            has_adjacent_solid = true;
                        } else {
                            has_adjacent_air = true;
                        }
                    } else {
                        has_adjacent_air = true;
                    }
                }

                if has_adjacent_air && has_adjacent_solid {
                    edge.insert(block_pos, (block, id));
                }
            }

            for (pos, (block, id)) in edge {
                let mesh = self.build_block_mesh(&block, pos, idx);

                if mesh.vertices.is_empty() {
                    continue;
                }

                let pos = section_pos + pos.as_vec3();
                let transform = Transform::from_translation(pos);

                meshes.push((mesh, transform, id));
            }
        }

        meshes
    }
}
