use bevy::{
    log::info,
    math::{IVec3, Vec3},
    prelude::Transform,
    utils::HashMap,
};
use gyra_proto::smp;

use super::block_builder::Block;

#[derive(Default, Debug, Clone)]
pub struct ChunkMesh {
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
}

impl<'a> ChunkConstructor<'a> {
    pub fn new(column: &'a smp::ChunkColumn) -> Self {
        Self {
            column,
            pos: IVec3::new(column.x as i32, 0, column.z as i32),
        }
    }

    fn block_at(&self, section: usize, pos: IVec3) -> Option<Block> {
        let column_section = self.column.chunks[section].as_ref()?;
        let id = column_section.block_id(pos.x as _, pos.y as _, pos.z as _);

        Some(Block::from_id(id))
    }

    fn build_block_mesh(&self, block: &Block, pos: IVec3, section: usize) -> ChunkMesh {
        // if block is air, return empty mesh
        // impl support for water
        if !block.shape().is_solid() {
            return ChunkMesh::default();
        }

        let mut vertices = Vec::with_capacity(24);
        let mut normals = Vec::with_capacity(24);
        let mut uv = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        const NATURAL_UV: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

        let base_index = vertices.len() as u32;

        let directions = [
            (
                IVec3::new(0, 0, 1),
                [0.0, 0.0, 1.0],
                [
                    // Front
                    [0.0, 0.0, 1.0],
                    [1.0, 0.0, 1.0],
                    [1.0, 1.0, 1.0],
                    [0.0, 1.0, 1.0],
                ],
            ),
            (
                IVec3::new(0, 0, -1),
                [0.0, 0.0, -1.0],
                [
                    // Back
                    [1.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [1.0, 1.0, 0.0],
                ],
            ),
            (
                IVec3::new(0, 1, 0),
                [0.0, 1.0, 0.0],
                [
                    // Top
                    [0.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 1.0, 0.0],
                ],
            ),
            (
                IVec3::new(0, -1, 0),
                [0.0, -1.0, 0.0],
                [
                    // Bottom
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 0.0, 1.0],
                    [0.0, 0.0, 1.0],
                ],
            ),
            (
                IVec3::new(-1, 0, 0),
                [-1.0, 0.0, 0.0],
                [
                    // Left
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 1.0, 1.0],
                    [0.0, 1.0, 0.0],
                ],
            ),
            (
                IVec3::new(1, 0, 0),
                [1.0, 0.0, 0.0],
                [
                    // Right
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
                    base_index + 2,
                    base_index + 3,
                    base_index,
                ]);
            };

            let adjacent_pos = pos + *offset;

            if let Some(adjacent_block) = self.block_at(section, adjacent_pos) {
                if adjacent_block.shape().is_solid() {
                    continue; // Skip this face if the adjacent block is solid
                }
            }

            add_face(face_vertices, *normal, &NATURAL_UV, idx);
        }

        ChunkMesh {
            vertices,
            normals,
            uv,
            indices,
        }
    }

    // /**
    //  * Construct the chunk mesh
    //  * Implement Vertices
    //  * Implement Normals
    //  * Implement UVs
    //  * Occlusion
    //  */
    // pub fn construct_hash(&mut self) -> Vec<(ChunkMesh, Transform)> {
    //     let mut meshes = Vec::new();

    //     let cull_directions = vec![
    //         IVec3::new(0, 0, 1),  // Front
    //         IVec3::new(0, 0, -1), // Back
    //         IVec3::new(0, 1, 0),  // Top
    //         IVec3::new(0, -1, 0), // Bottom
    //         IVec3::new(-1, 0, 0), // Left
    //         IVec3::new(1, 0, 0),  // Right
    //     ];

    //     for (idx, blocks) in &self.blocks {
    //         let section_pos = Vec3::new(self.pos.x as f32, *idx as f32, self.pos.z as f32) * 16.0;

    //         let blocks = blocks
    //             .into_iter()
    //             .filter(|(_, block)| block.shape().is_visible());

    //         // The check
    //         /*
    //          * considering this, we only need to render the y blocks, not the x blocks
    //          yyyy  we only need to render the y blocks
    //          yxxx  not the x blocks
    //          yxxx
    //          yxxx
    //         */
    //         let mut edge = HashMap::new();

    //         for (block_pos, block) in blocks {
    //             let mut has_adjacent_air = false;
    //             let mut has_adjacent_solid = false;

    //             for direction in cull_directions.iter() {
    //                 // we need to check if there is a air adjacent to the block
    //                 let adjacent_pos = *block_pos + *direction;

    //                 if let Some(adjacent_block) = self.block_at(*idx as _, adjacent_pos) {
    //                     if adjacent_block.shape().is_visible() {
    //                         has_adjacent_solid = true;
    //                     } else {
    //                         has_adjacent_air = true;
    //                     }
    //                 } else {
    //                     has_adjacent_air = true;
    //                 }
    //             }

    //             if has_adjacent_air && has_adjacent_solid {
    //                 edge.insert(block_pos, block);
    //             }
    //         }

    //         // info!("Processing {} blocks in edge.", edge.keys().len());

    //         for (pos, block) in edge {
    //             let mesh = self.build_block_mesh(block, *pos, *idx as _);

    //             if mesh.vertices.is_empty() {
    //                 continue;
    //             }

    //             let pos = pos.as_vec3() + section_pos;
    //             let transform = Transform::from_translation(pos);

    //             meshes.push((mesh, transform));
    //         }
    //     }

    //     info!("Rendered {} blocks in chunk.", meshes.len());
    //     meshes
    // }

    pub fn construct(&mut self) -> Vec<(ChunkMesh, Transform)> {
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
            .chunks
            .iter()
            .enumerate()
            .filter_map(|(idx, section)| Some((idx, section.as_ref()?)))
        {
            let blocks = (0..16).flat_map(|x| {
                (0..16).flat_map(move |y| {
                    (0..16).map(move |z| {
                        let pos = IVec3::new(x, y, z);
                        let block_id = section.block_id(x as _, y as _, z as _);
                        let block = Block::from_id(block_id);

                        (pos, block)
                    })
                })
            });

            let section_pos = Vec3::new(self.pos.x as f32, idx as f32, self.pos.z as f32) * 16.0;

            let blocks = blocks
                .into_iter()
                .filter(|(_, block)| block.shape().is_visible());

            // The check
            /*
             * considering this, we only need to render the y blocks, not the x blocks
             yyyy  we only need to render the y blocks
             yxxx  not the x blocks
             yxxx
             yxxx
            */

            let mut edge = HashMap::new();

            for (block_pos, block) in blocks {
                let mut has_adjacent_air = false;
                let mut has_adjacent_solid = false;

                for direction in cull_directions.iter() {
                    // we need to check if there is a air adjacent to the block
                    let adjacent_pos = block_pos + *direction;

                    if let Some(adjacent_block) = self.block_at(idx as _, adjacent_pos) {
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
                    edge.insert(block_pos, block);
                }
            }

            // info!("Processing {} blocks in edge.", edge.keys().len());

            for (pos, block) in edge {
                let mesh = self.build_block_mesh(&block, pos, idx as _);

                if mesh.vertices.is_empty() {
                    continue;
                }

                let pos = pos.as_vec3() + section_pos;
                let transform = Transform::from_translation(pos);

                meshes.push((mesh, transform));
            }
        }

        info!("Rendered {} blocks in chunk.", meshes.len());
        meshes
    }
}
