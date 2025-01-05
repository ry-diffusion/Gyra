use godot::prelude::*;
pub mod essentials;
pub mod gd;

struct GyraLibrary;

#[gdextension]
unsafe impl ExtensionLibrary for GyraLibrary {}
