use crate::essentials::gd::BindableCallable;
use crate::gd::Gyra;
use godot::classes::RichTextLabel;
use godot::obj::WithBaseField;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node3D)]
pub struct GyraPlayer {
    base: Base<Node3D>,
    camera: Gd<Camera3D>,
}

#[godot_api]
impl INode3D for GyraPlayer {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            /* No, I won't use an option for a thing that really exists. */
            camera: Camera3D::new_alloc(),
        }
    }

    fn ready(&mut self) {
        self.camera = self.base().get_node_as("Camera");
        let mut gyra = Gyra::singleton();

        gyra.connect("player_position_sync", &self.bind("player_position_sync"));

        godot_print_rich!("Now you are [i]Bound[/i] to the [b]Gyra[/b]! Enjoy.");
    }
}

#[godot_api]
impl GyraPlayer {
    #[func]
    fn player_position_sync(&mut self, pos: Vector3, yaw: f32, pitch: f32) {
        godot_print_rich!(
            "Hohoho. I'm a player! I'm at {:?}, yaw: {}, pitch: {}",
            pos,
            yaw,
            pitch
        );

        self.camera.set_rotation(Vector3::new(pitch, yaw, 0.0));
        self.base_mut().set_position(pos);

        let mut debug_position = self.base().get_node_as::<RichTextLabel>("%DebugPosition");
        debug_position.set_text(&format!("Pos: {}/{}/{}", pos.x, pos.y, pos.z));
    }
}
