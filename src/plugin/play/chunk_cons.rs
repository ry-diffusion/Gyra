/*
 * Greedy meshing constructor :3
*/
use super::block_builder::{Block, Shape};
use bevy::log::debug;
use bevy::utils::HashSet;
use bevy::{
    log::info,
    math::{IVec3, Vec3},
    prelude::Transform,
    utils::HashMap,
};
use gyra_proto::smp;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;
use std::iter::{Filter, FlatMap, Map};
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Face {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Default, Debug, Clone)]
pub struct QuadMesh {
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Quad {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

/* Thanks tantan for this! */
fn greedy(input: &mut [u16; 16], lod_size: u32) -> Vec<Quad> {
    let mut quads = Vec::new();

    for row in 0..input.len() {
        let mut y = 0;

        // Let's find the first non-empty cell
        while y < lod_size {
            y += (input[row] >> y).trailing_zeros();

            if y >= lod_size {
                continue;
            }

            let height = (input[row] >> y).trailing_ones();

            // let's find the height of the quad
            // This is basically 0b1 * height but hard.
            let height_mask = u16::checked_shl(1, height).map_or(!0, |x| x - 1);

            // the mask to clear the height
            let mask = height_mask << y;

            let mut width = 1;

            // let's grow the width
            while row + width < lod_size as usize && row + width < input.len() {
                let next_row_height = (input[row + width as usize] >> y) & height_mask;

                // we cant expand :D
                if next_row_height != height_mask {
                    break;
                }

                input[row + width] &= !mask;
                width += 1;
            }

            quads.push(Quad {
                x: row as u32,
                y,
                width: width as u32,
                height: height,
            });

            y += height;
        }
    }

    quads
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

    fn quads_to_mesh(by: Face, quads: &[Quad]) -> Vec<(QuadMesh, Transform)> {
        let mut meshes = Vec::with_capacity(quads.len());

        for quad in quads {
            let mut vertices = Vec::new();
            let mut normals = Vec::new();
            let mut uv = Vec::new();
            let mut indices = Vec::new();

            let mut add_face = |a: Vec3, b: Vec3, c: Vec3, d: Vec3| {
                let normal = (b - a).cross(c - a).normalize();

                vertices.push([a.x, a.y, a.z]);
                vertices.push([b.x, b.y, b.z]);
                vertices.push([c.x, c.y, c.z]);
                vertices.push([d.x, d.y, d.z]);

                normals.push([normal.x, normal.y, normal.z]);
                normals.push([normal.x, normal.y, normal.z]);
                normals.push([normal.x, normal.y, normal.z]);
                normals.push([normal.x, normal.y, normal.z]);

                uv.push([0.0, 0.0]);
                uv.push([1.0, 0.0]);
                uv.push([1.0, 1.0]);
                uv.push([0.0, 1.0]);

                let idx = vertices.len() as u32 - 4;

                indices.push(idx);
                indices.push(idx + 1);
                indices.push(idx + 2);

                indices.push(idx + 2);
                indices.push(idx + 3);
                indices.push(idx);
            };

            let x = quad.x as f32;
            let y = quad.y as f32;
            let w = quad.width as f32;
            let h = quad.height as f32;

            match by {
                Face::Top => {
                    add_face(
                        Vec3::new(x, y, quad.y as f32),
                        Vec3::new(x + w, y, quad.y as f32),
                        Vec3::new(x + w, y, quad.y as f32 + h),
                        Vec3::new(x, y, quad.y as f32 + h),
                    );
                }

                Face::Bottom => {
                    add_face(
                        Vec3::new(x, y, quad.y as f32),
                        Vec3::new(x + w, y, quad.y as f32),
                        Vec3::new(x + w, y, quad.y as f32 + h),
                        Vec3::new(x, y, quad.y as f32 + h),
                    );
                }

                Face::Left => {
                    add_face(
                        Vec3::new(x, y, quad.y as f32),
                        Vec3::new(x + w, y, quad.y as f32),
                        Vec3::new(x + w, y, quad.y as f32 + h),
                        Vec3::new(x, y, quad.y as f32 + h),
                    );
                }

                Face::Right => {
                    add_face(
                        Vec3::new(x, y, quad.y as f32),
                        Vec3::new(x + w, y, quad.y as f32),
                        Vec3::new(x + w, y, quad.y as f32 + h),
                        Vec3::new(x, y, quad.y as f32 + h),
                    );
                }
            }

            let mesh = QuadMesh {
                vertices,
                normals,
                uv,
                indices,
            };

            let transform = Transform::from_translation(Vec3::new(0.0, y, 0.0));

            meshes.push((mesh, transform));
        }

        meshes
    }

    /* Construct the chunk by the faces */
    fn build_quads(&mut self, shape: Shape, plane_2d: HashMap<IVec3, (Block, u16)>) -> Vec<Quad> {
        // the 2D Binary data
        // (0, 0) Bottom Left
        // True values are the {shape} shapes.
        let mut chunk_arr = [0u16; 16];

        /* Now we need to transform the plane_2d into a binary data */
        for (pos, (block, id)) in plane_2d {
            let x = pos.x as usize;
            let z = pos.z as usize;

            if block.shape() == shape {
                chunk_arr[z] |= 1 << x;
            }
        }

        greedy(&mut chunk_arr, 16)
    }

    fn get_visible_faces(looking_at: Vec3, pos: Vec3) -> Vec<Face> {
        let mut faces = Vec::new();
        let mut directions = [
            (Face::Top, Vec3::new(0.0, 1.0, 0.0)),
            (Face::Bottom, Vec3::new(0.0, -1.0, 0.0)),
            (Face::Left, Vec3::new(-1.0, 0.0, 0.0)),
            (Face::Right, Vec3::new(1.0, 0.0, 0.0)),
        ];

        for (face, direction) in directions.iter() {
            let dot = looking_at.dot(*direction);

            if dot > 0.0 {
                faces.push(*face);
            }
        }

        faces
    }

    fn slice_chunk_into_faces(
        &self,
        blocks: HashMap<IVec3, (Block, u16)>,
        section: usize,
    ) -> HashMap<Face, HashMap<IVec3, (Block, u16)>> {
        let mut faces = HashMap::new();
        let mut directions = [
            (Face::Top, IVec3::new(0, 1, 0)),
            (Face::Bottom, IVec3::new(0, -1, 0)),
            (Face::Left, IVec3::new(-1, 0, 0)),
            (Face::Right, IVec3::new(1, 0, 0)),
        ];

        for (pos, (block, id)) in blocks {
            let mut add_face =
                |face: &mut HashMap<IVec3, (Block, u16)>, pos: IVec3, block: Block, id: u16| {
                    face.insert(pos, (block, id));
                };

            for (face, direction) in directions.iter() {
                let adjacent_pos = pos + *direction;
                let storage = faces.entry(*face).or_insert(HashMap::new());

                if let Some(adjacent_block) = self.block_at(section, adjacent_pos) {
                    if !adjacent_block.shape().is_visible() {
                        add_face(storage, pos, block, id);
                    }
                } else {
                    // add_face(storage, pos, block, id);
                }
            }
        }

        faces
    }

    pub fn construct(&mut self, looking_at: Vec3) -> Vec<(QuadMesh, Transform, u16)> {
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
            debug!("Rendering section: {idx}");

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

            // Ambient occlusion
            let mut edge = HashMap::new();
            self.ao(cull_directions.as_slice(), idx, blocks, &mut edge);

            // how we can only get the top face?
            /*
             * we can only get the top face by checking if the block is not solid
             * and the block above is solid
             */

            let mut faces = self.slice_chunk_into_faces(edge.clone(), idx);

            // /* print lowest btm  coord */
            //
            // for (face, blocks) in &faces {
            //     let mut lowest = IVec3::new(123123, 123123, 13212);
            //
            //     if blocks.is_empty() {
            //         info!("Face {face:?} is empty!");
            //         continue;
            //     }
            //
            //     for (pos, _) in blocks {
            //         if pos.y < lowest.y {
            //             lowest = *pos;
            //         }
            //     }
            //
            //     info!("Lowest for {face:?}: \n > {lowest:?}");
            // }

            let mut r = (0, 0);

            for (face, blocks) in faces {
                if blocks.is_empty() {
                    continue;
                }

                let mut len_blocks = blocks.len();

                debug!("Face: {face:?} has {len_blocks}");

                // let's render the top face
                let mut quads = self.build_quads(Shape::Cube, blocks);
                let mut face = Self::quads_to_mesh(face, &quads);

                r.0 += len_blocks;
                r.1 += face.len();

                for (mesh, transform) in face {
                    let pos = section_pos + transform.translation;
                    let trans = Transform::from_translation(pos);

                    if mesh.vertices.is_empty() {
                        continue;
                    }

                    meshes.push((mesh, trans, 2));
                }
            }

            debug!(
                "Reduced %R from {} to {}, {:.2}% rendered!",
                r.0,
                r.1,
                (100.0 * r.1 as f32) / r.0 as f32
            );
        }

        meshes
    }

    fn ao(
        &mut self,
        cull_directions: &[IVec3],
        idx: usize,
        blocks: impl Iterator<Item = (IVec3, Block, u16)>,
        edge: &mut HashMap<IVec3, (Block, u16)>,
    ) {
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
    }
}
