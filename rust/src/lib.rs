use essentials::logger;
use godot::{classes::Engine, prelude::*};
pub mod essentials;
pub mod game;
pub mod gd;

struct GyraLibrary;

#[gdextension]
unsafe impl ExtensionLibrary for GyraLibrary {
    fn on_level_init(level: InitLevel) {
        if InitLevel::Scene == level {
            godot_print_rich!("[color=green]Gyra[/color] library initialized!");
            godot_print_rich!("Some back with another [i]Disguise[/i]!");
            logger::init(log::LevelFilter::Trace).unwrap();

            Engine::singleton().register_singleton("GyraSingleton", &gd::Gyra::new_alloc());
        }
    }

    fn on_level_deinit(level: InitLevel) {
        if level == InitLevel::Scene {
            let mut engine = Engine::singleton();
            if let Some(network_singleton) = engine.get_singleton("GyraSingleton") {
                network_singleton.free();
                engine.unregister_singleton("GyraSingleton");
            }
        }
    }
}
