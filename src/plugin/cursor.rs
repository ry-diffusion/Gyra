use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_cosmic_edit::CursorPluginDisabled;

#[derive(Debug, Resource)]
pub struct CursorState {
    pub is_locked: bool,
}

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, key_handler)
            .add_systems(PostUpdate, state_handler)
            .add_systems(PostUpdate, recenter)
            .add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands) {
    commands.insert_resource(CursorState { is_locked: false });
}

fn key_handler(keys: Res<ButtonInput<KeyCode>>, mut cursor_state: ResMut<CursorState>) {
    if keys.just_pressed(KeyCode::ControlRight) {
        info!("Toggling cursor lock");
        cursor_state.is_locked = !cursor_state.is_locked;
    }
}

fn state_handler(
    cursor_state: ResMut<CursorState>,
    mut primary_window: Single<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    if cursor_state.is_changed() {
        if cursor_state.is_locked {
            primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;
            primary_window.cursor_options.visible = false;
            commands.insert_resource(CursorPluginDisabled);
        } else {
            primary_window.cursor_options.grab_mode = CursorGrabMode::None;
            primary_window.cursor_options.visible = true;
            commands.remove_resource::<CursorPluginDisabled>();
        }
    }
}

fn recenter(
    mut primary_window: Single<&mut Window, With<PrimaryWindow>>,
    cursor_state: Res<CursorState>,
) {
    if !cursor_state.is_locked {
        return;
    }

    let center = Vec2::new(primary_window.width() / 2.0, primary_window.height() / 2.0);

    primary_window.set_cursor_position(Some(center));
}
