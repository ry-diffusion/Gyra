use std::collections::HashSet;
use crate::essentials::gd::BindableCallable;
use crate::gd::Gyra;
use godot::classes::{BoxMesh, CsgBox3D, MeshInstance3D, MultiMesh, MultiMeshInstance3D};
use godot::classes::multi_mesh::TransformFormat;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct GyraWorld {
    base: Base<Node3D>,
    chunks: Gd<Node3D>,
}

#[godot_api]
impl INode3D for GyraWorld {
    fn init(owner: Base<Self::Base>) -> Self {
        Self {
            base: owner,
            chunks: Node3D::new_alloc(),
        }
    }

    fn ready(&mut self) {
        let mut chunks = Node3D::new_alloc();
        chunks.set_name("Chunks");

        self.base_mut().add_child(&chunks);
        self.chunks.queue_free();

        let mut chunks: Gd<Node3D> = self.base_mut().get_node_as("Chunks");

        chunks.set_owner(&*self.base());

        self.chunks = chunks;

        Gyra::singleton().connect("show_chunks", &self.bind("on_show_chunks"));
    }
}

#[godot_api]
impl GyraWorld {
    #[func]
    fn on_show_chunks(&mut self, mut chunks: Vec<Gd<MeshInstance3D>>, chk_x: i32, chk_z: i32) {
        if (self
            .chunks
            .has_node(&format!("./Chunk_{}_{}", chk_x, chk_z)))
        {
            godot_print_rich!("Removing chunks at {}, {}", chk_x, chk_z);
            let chunks = self
                .chunks
                .get_node_as::<Node3D>(&format!("Chunk_{}_{}", chk_x, chk_z));

            chunks.free();
        }

        godot_print_rich!("Showing chunks at {}, {}", chk_x, chk_z);

        let mut chunk = MultiMeshInstance3D::new_alloc();
        let mut chunk_mesh = MultiMesh::new_gd();
        
        chunk_mesh.set_transform_format(TransformFormat::TRANSFORM_3D);
        chunk_mesh.set_instance_count(chunks.len() as i32);
        
        let mut cube_mesh = BoxMesh::new_gd();
        cube_mesh.set_size(Vector3::new(1.0, 1.0, 1.0));
        chunk_mesh.set_mesh(&cube_mesh);
        
        chunk.set_multimesh(&chunk_mesh);
        
        chunk.set_name(&format!("Chunk_{}_{}", chk_x, chk_z));


        for (idx, cube) in chunks.iter_mut().enumerate() {
            cube.set_name(&format!("Cube number {}", idx));
            chunk.add_child(&*cube);
            cube.set_owner(&chunk);
            chunk_mesh.set_instance_transform(idx as i32, cube.get_transform());
        }

        self.chunks.add_child(&chunk);
        chunk.set_owner(&self.chunks);
    }
}
